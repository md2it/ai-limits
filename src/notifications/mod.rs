use std::collections::HashSet;
use std::io;

use crate::presentation::{is_limit_shown_to_user, limit_type_label, TimeContext};
use crate::types::{LimitInfo, SourceReport, StructuredSourceInfo};

mod content;
mod kinds;
mod store;

pub use content::Notification;
pub use kinds::{LimitNotificationKind, NotificationColor};
pub use store::{FileRemainingStore, PreviousRemainingStore};

pub trait NotificationDelivery {
    fn deliver(&self, notification: &Notification) -> io::Result<()>;
}

pub fn send_for_report_with_delivery(
    report: &SourceReport,
    sent: &mut HashSet<String>,
    store: &dyn PreviousRemainingStore,
    delivery: &impl NotificationDelivery,
) {
    for notification in notifications_for_report(report, store) {
        // `always_deliver` candidates (100% again) are edge-triggered: the
        // persistent store already confirmed this is a fresh transition, so
        // the process-lifetime `sent` set must not suppress a later, genuine
        // repeat of the same transition behind a static dedupe key.
        let should_deliver =
            notification.always_deliver || sent.insert(notification.dedupe_key.clone());
        if should_deliver {
            let _ = delivery.deliver(&notification);
        }
    }
}

pub fn notifications_for_report(
    report: &SourceReport,
    store: &dyn PreviousRemainingStore,
) -> Vec<Notification> {
    notifications_for_structured(&report.data.structured, store)
}

pub fn notifications_for_structured(
    info: &StructuredSourceInfo,
    store: &dyn PreviousRemainingStore,
) -> Vec<Notification> {
    if !info.status.access_available || !info.status.data_available {
        return Vec::new();
    }

    let time_context = TimeContext::from_structured(info);

    info.limits
        .iter()
        .filter(|limit| is_limit_shown_to_user(limit))
        .flat_map(|limit| notifications_for_limit(info, limit, &time_context, store))
        .collect()
}

fn notifications_for_limit(
    info: &StructuredSourceInfo,
    limit: &LimitInfo,
    time_context: &TimeContext,
    store: &dyn PreviousRemainingStore,
) -> Vec<Notification> {
    let Some(remaining) = kinds::remaining_percent(limit) else {
        return Vec::new();
    };

    let mut notifications = Vec::new();

    if let Some(kind) = kinds::matching_kind(remaining) {
        notifications.push(Notification::limit(
            &info.provider,
            &info.source,
            &limit.name,
            kind,
            remaining,
            limit.resets_at.as_deref(),
            time_context,
        ));
    }

    if is_replenished_transition(&info.provider, &limit.name, remaining, store) {
        notifications.push(Notification::replenished(
            &info.provider,
            &limit.name,
            limit.resets_at.as_deref(),
            time_context,
        ));
    }

    notifications
}

/// Reports the current remaining percent to the shared previous-remaining
/// store (keyed by provider + limit name, not source, so multiple sources for
/// the same limit share one transition) and returns whether this snapshot is
/// an exact return to 100% after a stored value below 100%.
fn is_replenished_transition(
    provider: &str,
    limit_name: &str,
    remaining: f64,
    store: &dyn PreviousRemainingStore,
) -> bool {
    let key = format!("{provider}|{}", limit_type_label(limit_name));
    let previous = store.replace(&key, remaining);

    // `remaining` is already clamped to at most 100.0, so `>=` is an exact
    // equality check against 100 without tripping a float-equality lint.
    remaining >= 100.0 && matches!(previous, Some(previous) if previous < 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifications::store::InMemoryRemainingStore;
    use crate::types::{AccountInfo, LimitInfo, Source, SourceData, SourceStatus, UsageInfo};
    use std::cell::Cell;

    fn no_previous() -> InMemoryRemainingStore {
        InMemoryRemainingStore::new()
    }

    fn structured_with_limit(remaining_percent: Option<f64>) -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_local".to_string(),
            source_link: String::new(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
                cli_authorization: None,
            },
            raw_data_available: false,
            collected_at: None,
            data_as_of: None,
            account: AccountInfo::default(),
            limits: vec![LimitInfo {
                name: "5h".to_string(),
                remaining_percent,
                ..Default::default()
            }],
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn creates_notification_for_threshold_remaining_percent() {
        let notifications =
            notifications_for_structured(&structured_with_limit(Some(75.0)), &no_previous());

        assert_eq!(
            notifications,
            vec![Notification {
                dedupe_key: "Codex|codex_local|5h|75".to_string(),
                title: "🟢 AI Limits".to_string(),
                subtitle: "Codex 5h - 75% left".to_string(),
                message: "reset unknown".to_string(),
                color: Some(NotificationColor::Green),
                always_deliver: false,
            }]
        );
    }

    #[test]
    fn creates_notification_when_remaining_is_below_threshold() {
        let notifications =
            notifications_for_structured(&structured_with_limit(Some(74.0)), &no_previous());

        assert_eq!(notifications[0].color, Some(NotificationColor::Green));
        assert_eq!(notifications[0].title, "🟢 AI Limits");
        assert_eq!(notifications[0].subtitle, "Codex 5h - 74% left");
        assert_eq!(notifications[0].message, "reset unknown");
    }

    #[test]
    fn dedupe_key_uses_threshold_not_exact_remaining_percent() {
        let first =
            notifications_for_structured(&structured_with_limit(Some(75.0)), &no_previous());
        let second =
            notifications_for_structured(&structured_with_limit(Some(74.0)), &no_previous());

        assert_eq!(first[0].dedupe_key, second[0].dedupe_key);
    }

    #[test]
    fn ignores_remaining_above_first_threshold() {
        assert!(
            notifications_for_structured(&structured_with_limit(Some(76.0)), &no_previous())
                .is_empty()
        );
    }

    #[test]
    fn derives_remaining_percent_from_used_percent() {
        let mut info = structured_with_limit(None);
        info.limits[0].used_percent = Some(50.0);

        let notifications = notifications_for_structured(&info, &no_previous());

        assert_eq!(notifications[0].color, Some(NotificationColor::Yellow));
        assert_eq!(notifications[0].title, "🟡 AI Limits");
        assert_eq!(notifications[0].subtitle, "Codex 5h - 50% left");
    }

    #[test]
    fn formats_notification_reset_with_shared_time_display() {
        let mut info = structured_with_limit(Some(50.0));
        info.collected_at = Some("2026-06-30T20:00:00Z".to_string());
        info.limits[0].resets_at = Some("2026-06-30T20:41:00Z".to_string());

        let notifications = notifications_for_structured(&info, &no_previous());

        assert!(notifications[0].message.starts_with("reset "));
        assert_ne!(notifications[0].message, "reset unknown");
        assert!(!notifications[0].message.contains("UTC"));
        assert!(!notifications[0].message.contains('T'));
        assert!(!notifications[0].message.ends_with('Z'));
    }

    #[test]
    fn ignores_unavailable_data() {
        let mut info = structured_with_limit(Some(25.0));
        info.status.data_available = false;

        assert!(notifications_for_structured(&info, &no_previous()).is_empty());
    }

    #[test]
    fn send_for_report_dedupes_within_session() {
        struct CountingDelivery(Cell<usize>);

        impl NotificationDelivery for CountingDelivery {
            fn deliver(&self, _notification: &Notification) -> io::Result<()> {
                self.0.set(self.0.get() + 1);
                Ok(())
            }
        }

        let report = SourceReport {
            source: Source::CodexLocal,
            data: SourceData {
                raw: None,
                structured: structured_with_limit(Some(75.0)),
                stderr: String::new(),
            },
        };
        let mut sent = HashSet::new();
        let store = no_previous();
        let delivery = CountingDelivery(Cell::new(0));

        send_for_report_with_delivery(&report, &mut sent, &store, &delivery);
        assert_eq!(sent.len(), 1);
        assert_eq!(delivery.0.get(), 1);

        send_for_report_with_delivery(&report, &mut sent, &store, &delivery);
        assert_eq!(sent.len(), 1);
        assert_eq!(delivery.0.get(), 1);
    }

    #[test]
    fn fires_replenished_again_after_a_new_deplete_to_100_cycle() {
        use std::cell::RefCell;

        struct RecordingDelivery(RefCell<Vec<Notification>>);

        impl NotificationDelivery for RecordingDelivery {
            fn deliver(&self, notification: &Notification) -> io::Result<()> {
                self.0.borrow_mut().push(notification.clone());
                Ok(())
            }
        }

        fn report_with_remaining(remaining: f64) -> SourceReport {
            SourceReport {
                source: Source::CodexLocal,
                data: SourceData {
                    raw: None,
                    structured: structured_with_limit(Some(remaining)),
                    stderr: String::new(),
                },
            }
        }

        let mut sent = HashSet::new();
        let store = no_previous();
        let delivery = RecordingDelivery(RefCell::new(Vec::new()));

        // 40 -> 100 -> 40 -> 100: two independent deplete/replenish cycles in
        // one continuous session, using the real dedupe path (sent + store).
        for remaining in [40.0, 100.0, 40.0, 100.0] {
            send_for_report_with_delivery(
                &report_with_remaining(remaining),
                &mut sent,
                &store,
                &delivery,
            );
        }

        let delivered = delivery.0.borrow();
        let replenished_count = delivered
            .iter()
            .filter(|notification| notification.subtitle.ends_with("100% again"))
            .count();
        assert_eq!(
            replenished_count, 2,
            "each deplete-then-100 cycle must notify again, not just the first"
        );
    }

    #[test]
    fn evaluates_source_report_structured_data() {
        let report = SourceReport {
            source: Source::CodexLocal,
            data: SourceData {
                raw: None,
                structured: structured_with_limit(Some(10.0)),
                stderr: String::new(),
            },
        };

        assert_eq!(
            notifications_for_report(&report, &no_previous())[0].color,
            Some(NotificationColor::Red)
        );
    }

    fn replenished(notifications: &[Notification]) -> Option<&Notification> {
        notifications
            .iter()
            .find(|notification| notification.subtitle.ends_with("100% again"))
    }

    #[test]
    fn fires_replenished_when_previous_below_100_and_current_exactly_100() {
        let store = InMemoryRemainingStore::new().seed("codex|5h", 40.0);

        let notifications =
            notifications_for_structured(&structured_with_limit(Some(100.0)), &store);

        let notification = replenished(&notifications).expect("should fire 100% again");
        assert_eq!(notification.title, "🔔 AI Limits");
        assert_eq!(notification.subtitle, "Codex 5h - 100% again");
        assert_eq!(notification.color, None);
        assert_eq!(notification.dedupe_key, "Codex|5h|100-again");
    }

    #[test]
    fn does_not_fire_replenished_on_cold_start_with_no_previous_value() {
        let notifications =
            notifications_for_structured(&structured_with_limit(Some(100.0)), &no_previous());

        assert!(replenished(&notifications).is_none());
    }

    #[test]
    fn does_not_fire_replenished_when_previous_is_already_100() {
        let store = InMemoryRemainingStore::new().seed("codex|5h", 100.0);

        let notifications =
            notifications_for_structured(&structured_with_limit(Some(100.0)), &store);

        assert!(replenished(&notifications).is_none());
    }

    #[test]
    fn does_not_fire_replenished_on_partial_rise_below_100() {
        let store = InMemoryRemainingStore::new().seed("codex|5h", 40.0);

        let notifications =
            notifications_for_structured(&structured_with_limit(Some(97.0)), &store);

        assert!(replenished(&notifications).is_none());
    }

    #[test]
    fn updates_stored_previous_after_evaluating_a_successful_snapshot() {
        let store = InMemoryRemainingStore::new();

        notifications_for_structured(&structured_with_limit(Some(40.0)), &store);
        assert_eq!(store.replace("codex|5h", 40.0), Some(40.0));
    }

    #[test]
    fn unsuccessful_snapshot_does_not_clear_or_rewrite_stored_value() {
        let store = InMemoryRemainingStore::new().seed("codex|5h", 40.0);
        let mut info = structured_with_limit(Some(100.0));
        info.status.data_available = false;

        notifications_for_structured(&info, &store);

        assert_eq!(store.replace("codex|5h", 999.0), Some(40.0));
    }

    #[test]
    fn replenished_key_is_shared_across_sources_for_the_same_provider_and_limit() {
        let store = InMemoryRemainingStore::new().seed("codex|5h", 40.0);

        let mut first_source = structured_with_limit(Some(100.0));
        first_source.source = "codex_local".to_string();
        let first = notifications_for_structured(&first_source, &store);
        assert!(replenished(&first).is_some());

        let mut second_source = structured_with_limit(Some(100.0));
        second_source.source = "codex_rpc".to_string();
        let second = notifications_for_structured(&second_source, &store);
        assert!(
            replenished(&second).is_none(),
            "second source must not re-fire the same provider+limit transition"
        );
    }
}
