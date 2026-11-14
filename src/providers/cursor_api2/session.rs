use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, OnceLock,
};

use serde_json::{json, Value};

use super::credentials::{Credential, CredentialSource, OsStore, Store};
use super::transport::{classify, Curl, Failure, Response, Transport};

static EPOCH: AtomicU64 = AtomicU64::new(0);
thread_local! { static COLLECTION_EPOCH: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }
pub fn collection_epoch() -> u64 {
    COLLECTION_EPOCH.get()
}
pub fn publish_if_current<R>(epoch: u64, publish: impl FnOnce() -> R) -> Option<R> {
    let _guard = state().lock().ok()?;
    (cache_epoch() == epoch).then(publish)
}
fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(State::default()))
}
pub fn cache_epoch() -> u64 {
    EPOCH.load(Ordering::SeqCst)
}
pub fn select_source(source: &str) -> Result<(), &'static str> {
    select_source_with(source, || ())
}
pub fn select_source_with<R>(source: &str, changed: impl FnOnce() -> R) -> Result<R, &'static str> {
    let source = CredentialSource::parse(source)?;
    let mut state = state().lock().map_err(|_| "Cursor session is busy")?;
    *state = State {
        chosen: Some(source),
        ..State::default()
    };
    EPOCH.fetch_add(1, Ordering::SeqCst);
    Ok(changed())
}
pub fn retain_snapshot(message: &str) -> bool {
    message == Failure::Transient.message()
        || message == Failure::Unreadable.message()
        || message == Failure::Malformed.message()
        || message == Failure::Forbidden.message()
}

#[derive(Default)]
struct State {
    current: Option<Credential>,
    chosen: Option<CredentialSource>,
    cooldown: i64,
    rejected: Option<String>,
    needs_choice: bool,
}

impl State {
    fn discover(&mut self, store: &mut impl Store) -> Result<(), Failure> {
        if self.needs_choice {
            return Err(Failure::Choice);
        }
        let ide = store.read(CredentialSource::Ide);
        let cli = store.read(CredentialSource::Cli);
        let selected = if let Some(source) = self
            .chosen
            .or_else(|| self.current.as_ref().map(|c| c.source))
        {
            let (selected, alternate) = if source == CredentialSource::Ide {
                (ide, cli)
            } else {
                (cli, ide)
            };
            match selected {
                Ok(c) => Ok(c),
                Err(error @ (Failure::Missing | Failure::Protected)) => {
                    match (self.current.as_ref(), alternate) {
                        (Some(current), Ok(other))
                            if current.same_account(&other) && self.chosen.is_none() =>
                        {
                            Ok(other)
                        }
                        _ => Err(error),
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            match (ide, cli) {
                (Ok(ide), Ok(cli)) if !ide.same_account(&cli) => Err(Failure::Choice),
                (Ok(ide), _) => Ok(ide),
                (
                    Err(Failure::Missing | Failure::SqliteUnavailable | Failure::Protected),
                    Ok(cli),
                ) => Ok(cli),
                (Err(a), Err(b)) => Err(if a == Failure::Missing { b } else { a }),
                (Err(e), Ok(_)) => Err(e),
            }
        };
        let selected = match selected {
            Ok(c) => c,
            Err(e) => {
                if e.invalidates_cache() && self.current.take().is_some() {
                    EPOCH.fetch_add(1, Ordering::SeqCst);
                }
                return Err(e);
            }
        };
        if self.rejected.as_ref() == Some(&selected.access) {
            if self.chosen.is_none() {
                let alternate_source = if selected.source == CredentialSource::Ide {
                    CredentialSource::Cli
                } else {
                    CredentialSource::Ide
                };
                if let Ok(alternate) = store.read(alternate_source) {
                    if selected.same_account(&alternate) {
                        self.current = Some(alternate);
                        return Ok(());
                    }
                }
            }
            return Err(Failure::SignedOut);
        }
        if let Some(current) = &self.current {
            if current.access == selected.access {
                return Ok(());
            }
            if !current.same_account(&selected) {
                self.current = None;
                self.needs_choice = true;
                self.cooldown = 0;
                self.rejected = None;
                EPOCH.fetch_add(1, Ordering::SeqCst);
                return Err(Failure::Choice);
            }
            // Cursor may still store the old token while this process owns a newer session.
            if current
                .expires
                .zip(selected.expires)
                .is_some_and(|(a, b)| a >= b)
            {
                return Ok(());
            }
        }
        self.current = Some(selected);
        self.rejected = None;
        Ok(())
    }
    fn reject(&mut self) {
        self.rejected = self.current.as_ref().map(|c| c.access.clone());
        self.current = None;
        EPOCH.fetch_add(1, Ordering::SeqCst);
    }
}

pub(super) fn with_session<R>(
    run: impl FnOnce(&mut Session<'_, Curl, OsStore>) -> Result<R, Failure>,
) -> Result<R, Failure> {
    let mut state = state().lock().map_err(|_| Failure::Transient)?;
    let mut transport = Curl;
    let mut store = OsStore;
    let mut session = Session {
        state: &mut state,
        transport: &mut transport,
        store: &mut store,
        refreshed: false,
        reloaded: false,
        now: chrono::Utc::now().timestamp(),
    };
    let result = session.begin().and_then(|()| {
        let result = run(&mut session);
        if result.is_ok() {
            session.finish()?;
        }
        result
    });
    COLLECTION_EPOCH.set(cache_epoch());
    result
}

pub(super) struct Session<'a, T, S> {
    state: &'a mut State,
    transport: &'a mut T,
    store: &'a mut S,
    refreshed: bool,
    reloaded: bool,
    now: i64,
}

impl<T: Transport, S: Store> Session<'_, T, S> {
    fn begin(&mut self) -> Result<(), Failure> {
        self.state.discover(self.store)?;
        if self.state.cooldown > self.now {
            return Err(Failure::Transient);
        }
        let current = self.state.current.as_ref().ok_or(Failure::Missing)?;
        if current.expires.is_some_and(|exp| exp <= self.now + 300) && current.refresh.is_some() {
            let valid = current.expires.is_some_and(|exp| exp > self.now);
            if let Err(e) = self.refresh() {
                if !valid || e != Failure::Transient {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Failure> {
        let current = self.state.current.as_ref().ok_or(Failure::Missing)?.clone();
        match self.store.read(current.source) {
            Ok(latest) if latest.access == current.access || current.same_account(&latest) => {
                Ok(())
            }
            Ok(_) => {
                self.state.reject();
                self.state.needs_choice = true;
                Err(Failure::Choice)
            }
            Err(Failure::Missing) => {
                self.state.reject();
                Err(Failure::SignedOut)
            }
            Err(error) => Err(error),
        }
    }

    fn send(&mut self, method: &str, body: &str) -> Result<Response, Failure> {
        let token = self
            .state
            .current
            .as_ref()
            .ok_or(Failure::Missing)?
            .access
            .as_str();
        match self.transport.send(method, body, Some(token)) {
            Ok(response) => {
                if response.status == 429 || response.status >= 500 {
                    self.cooldown(response.retry_after);
                }
                Ok(response)
            }
            Err(e) => {
                self.cooldown(None);
                Err(e)
            }
        }
    }
    fn cooldown(&mut self, seconds: Option<u64>) {
        self.state.cooldown = self
            .now
            .saturating_add(seconds.unwrap_or(60).max(60).min(i64::MAX as u64) as i64);
    }

    pub fn post(&mut self, method: &str, body: &str) -> Result<String, Failure> {
        let response = self.send(method, body)?;
        match classify(&response) {
            Ok(_) => return Ok(response.body),
            Err(Failure::Unauthorized) => {}
            Err(e) => return Err(e),
        }
        if !self.reloaded {
            self.reloaded = true;
            let old = self.state.current.as_ref().map(|c| c.access.clone());
            self.state.discover(self.store)?;
            if old != self.state.current.as_ref().map(|c| c.access.clone()) {
                let response = self.send(method, body)?;
                match classify(&response) {
                    Ok(_) => return Ok(response.body),
                    Err(Failure::Unauthorized) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        if !self.refreshed && self.state.cooldown <= self.now {
            self.refresh()?;
            let response = self.send(method, body)?;
            match classify(&response) {
                Ok(_) => return Ok(response.body),
                Err(Failure::Unauthorized) => {}
                Err(e) => return Err(e),
            }
        }
        if self.state.cooldown > self.now {
            return Err(Failure::Transient);
        }
        // An API rejection is not proof the vendor session has been explicitly revoked.
        Err(Failure::Unauthorized)
    }

    fn refresh(&mut self) -> Result<(), Failure> {
        self.refreshed = true;
        let credential = self.state.current.as_ref().ok_or(Failure::Missing)?.clone();
        let refresh = credential.refresh.as_deref().ok_or(Failure::Unauthorized)?;
        let body = json!({"grant_type":"refresh_token","client_id":"KbZUR41cY7W6zRSdpSUJ7I7mLYBKOCmB","refresh_token":refresh}).to_string();
        let response = match self.transport.send("oauth/token", &body, None) {
            Ok(r) => r,
            Err(e) => {
                self.cooldown(None);
                return Err(e);
            }
        };
        if response.status == 429 || response.status >= 500 {
            self.cooldown(response.retry_after);
            return Err(Failure::Transient);
        }
        let value: Value = serde_json::from_str(&response.body).map_err(|_| {
            self.cooldown(None);
            Failure::Transient
        })?;
        if (200..300).contains(&response.status)
            && value.get("shouldLogout").and_then(Value::as_bool) == Some(true)
        {
            if self.state.chosen.is_none() {
                let alternate_source = if credential.source == CredentialSource::Ide {
                    CredentialSource::Cli
                } else {
                    CredentialSource::Ide
                };
                if let Ok(alternate) = self.store.read(alternate_source) {
                    if credential.same_account(&alternate)
                        && alternate.expires.is_none_or(|expires| expires > self.now)
                    {
                        self.state.rejected = Some(credential.access);
                        self.state.current = Some(alternate);
                        return Ok(());
                    }
                }
            }
            self.state.reject();
            return Err(Failure::SignedOut);
        }
        if !(200..300).contains(&response.status) {
            self.cooldown(None);
            return Err(Failure::Transient);
        }
        let access = value
            .get("access_token")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                self.cooldown(None);
                Failure::Transient
            })?;
        let refresh = value
            .get("refresh_token")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or(access);
        let updated = Credential::new(credential.source, access.into(), Some(refresh.into()))?;
        if !credential.same_account(&updated) {
            self.state.reject();
            self.state.needs_choice = true;
            return Err(Failure::Choice);
        }
        // Re-read before adoption so an external account switch cannot attach the old response to a new account.
        let latest = self.store.read(credential.source)?;
        if !credential.same_account(&latest) {
            self.state.reject();
            self.state.needs_choice = true;
            return Err(Failure::Choice);
        }
        self.state.current = Some(updated);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    fn token(exp: i64) -> String {
        // The payload's subject is fixed; these tokens never leave the fake transport.
        let payload = format!("{{\"sub\":\"test\",\"exp\":{exp}}}");
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut encoded = String::new();
        let mut bits = 0u32;
        let mut count = 0;
        for b in payload.bytes() {
            bits = (bits << 8) | u32::from(b);
            count += 8;
            while count >= 6 {
                count -= 6;
                encoded.push(alphabet[((bits >> count) & 63) as usize] as char);
            }
        }
        if count > 0 {
            encoded.push(alphabet[((bits << (6 - count)) & 63) as usize] as char);
        }
        format!("x.{encoded}.x")
    }
    fn credential(exp: i64) -> Credential {
        Credential::new(CredentialSource::Ide, token(exp), Some("refresh".into())).unwrap()
    }
    struct FakeStore {
        credential: Credential,
    }
    impl Store for FakeStore {
        fn read(&mut self, source: CredentialSource) -> Result<Credential, Failure> {
            if source == CredentialSource::Ide {
                Ok(self.credential.clone())
            } else {
                Err(Failure::Missing)
            }
        }
    }
    struct FakeTransport {
        responses: VecDeque<Response>,
        calls: Vec<String>,
    }
    impl Transport for FakeTransport {
        fn send(
            &mut self,
            method: &str,
            _body: &str,
            _token: Option<&str>,
        ) -> Result<Response, Failure> {
            self.calls.push(method.into());
            self.responses.pop_front().ok_or(Failure::Transient)
        }
    }
    fn response(status: u16, body: Value) -> Response {
        Response {
            status,
            body: body.to_string(),
            retry_after: None,
        }
    }
    fn exercise(
        state: &mut State,
        store: &mut FakeStore,
        transport: &mut FakeTransport,
        run: impl FnOnce(&mut Session<'_, FakeTransport, FakeStore>) -> Result<(), Failure>,
    ) -> Result<(), Failure> {
        let mut session = Session {
            state,
            store,
            transport,
            refreshed: false,
            reloaded: false,
            now: 1000,
        };
        session.begin()?;
        run(&mut session)
    }
    #[test]
    fn expired_token_refreshes_and_survives_old_store_reread() {
        let mut state = State::default();
        let mut store = FakeStore {
            credential: credential(900),
        };
        let mut transport = FakeTransport {
            responses: VecDeque::from([
                response(200, json!({"access_token":token(9999)})),
                response(200, json!({"planInfo":{}})),
                response(200, json!({"planInfo":{}})),
            ]),
            calls: vec![],
        };
        exercise(&mut state, &mut store, &mut transport, |s| {
            s.post("GetPlanInfo", "{}").map(|_| ())
        })
        .unwrap();
        exercise(&mut state, &mut store, &mut transport, |s| {
            s.post("GetPlanInfo", "{}").map(|_| ())
        })
        .unwrap();
        assert_eq!(
            transport.calls,
            ["oauth/token", "GetPlanInfo", "GetPlanInfo"]
        );
        assert_eq!(state.current.as_ref().unwrap().access, token(9999));
    }
    #[test]
    fn one_refresh_budget_includes_later_methods() {
        let mut state = State::default();
        let mut store = FakeStore {
            credential: credential(9999),
        };
        let mut transport = FakeTransport {
            responses: VecDeque::from([
                response(401, json!({})),
                response(200, json!({"access_token":token(19999)})),
                response(200, json!({})),
                response(401, json!({})),
            ]),
            calls: vec![],
        };
        let result = exercise(&mut state, &mut store, &mut transport, |s| {
            s.post("GetPlanInfo", "{}")?;
            s.post("GetHardLimit", "{}").map(|_| ())
        });
        assert_eq!(result, Err(Failure::Unauthorized));
        assert_eq!(
            transport
                .calls
                .iter()
                .filter(|s| *s == "oauth/token")
                .count(),
            1
        );
    }
    #[test]
    fn forbidden_does_not_refresh() {
        let mut state = State::default();
        let mut store = FakeStore {
            credential: credential(9999),
        };
        let mut transport = FakeTransport {
            responses: VecDeque::from([response(403, json!({}))]),
            calls: vec![],
        };
        assert_eq!(
            exercise(&mut state, &mut store, &mut transport, |s| s
                .post("GetPlanInfo", "{}")
                .map(|_| ())),
            Err(Failure::Forbidden)
        );
        assert_eq!(transport.calls.len(), 1);
    }
    #[test]
    fn proactive_transient_failure_keeps_still_valid_token_and_cooldown() {
        let mut state = State::default();
        let mut store = FakeStore {
            credential: credential(1100),
        };
        let mut transport = FakeTransport {
            responses: VecDeque::from([response(500, json!({})), response(200, json!({}))]),
            calls: vec![],
        };
        exercise(&mut state, &mut store, &mut transport, |s| {
            s.post("GetPlanInfo", "{}").map(|_| ())
        })
        .unwrap();
        assert_eq!(state.cooldown, 1060);
        assert_eq!(
            exercise(&mut state, &mut store, &mut transport, |_| Ok(())),
            Err(Failure::Transient)
        );
        assert_eq!(transport.calls.len(), 2);
    }
    #[test]
    fn rate_limit_prevents_repeated_manual_network_attempts() {
        let mut state = State::default();
        let mut store = FakeStore {
            credential: credential(9999),
        };
        let mut r = response(429, json!({}));
        r.retry_after = Some(120);
        let mut transport = FakeTransport {
            responses: VecDeque::from([r]),
            calls: vec![],
        };
        assert_eq!(
            exercise(&mut state, &mut store, &mut transport, |s| s
                .post("GetPlanInfo", "{}")
                .map(|_| ())),
            Err(Failure::Transient)
        );
        assert_eq!(state.cooldown, 1120);
        assert_eq!(
            exercise(&mut state, &mut store, &mut transport, |_| Ok(())),
            Err(Failure::Transient)
        );
        assert_eq!(transport.calls.len(), 1);
    }
    #[test]
    fn terminal_rejection_clears_session_and_does_not_retry_same_store() {
        let mut state = State::default();
        let mut store = FakeStore {
            credential: credential(900),
        };
        let mut transport = FakeTransport {
            responses: VecDeque::from([response(200, json!({"shouldLogout":true}))]),
            calls: vec![],
        };
        assert_eq!(
            exercise(&mut state, &mut store, &mut transport, |_| Ok(())),
            Err(Failure::SignedOut)
        );
        assert!(state.current.is_none());
        assert_eq!(
            exercise(&mut state, &mut store, &mut transport, |_| Ok(())),
            Err(Failure::SignedOut)
        );
        assert_eq!(transport.calls.len(), 1);
    }
    #[test]
    fn missing_refresh_does_not_invalidate_accepted_access() {
        let mut state = State::default();
        let mut c = credential(900);
        c.refresh = None;
        let mut store = FakeStore { credential: c };
        let mut transport = FakeTransport {
            responses: VecDeque::from([response(200, json!({}))]),
            calls: vec![],
        };
        exercise(&mut state, &mut store, &mut transport, |s| {
            s.post("GetPlanInfo", "{}").map(|_| ())
        })
        .unwrap();
        assert_eq!(transport.calls, ["GetPlanInfo"]);
    }
    #[test]
    fn unreadable_refresh_payload_never_signs_out() {
        let mut state = State::default();
        let mut store = FakeStore {
            credential: credential(900),
        };
        let mut transport = FakeTransport {
            responses: VecDeque::from([Response {
                status: 200,
                body: "invalid".into(),
                retry_after: None,
            }]),
            calls: vec![],
        };
        assert_eq!(
            exercise(&mut state, &mut store, &mut transport, |_| Ok(())),
            Err(Failure::Transient)
        );
        assert!(state.current.is_some());
    }
    #[test]
    fn account_change_stays_blocked_until_explicit_choice() {
        let mut state = State {
            current: Some(credential(9999)),
            ..State::default()
        };
        let mut changed = credential(19999);
        changed.subject = Some("other".into());
        let mut store = FakeStore {
            credential: changed,
        };
        assert_eq!(state.discover(&mut store), Err(Failure::Choice));
        assert!(state.current.is_none());
        assert_eq!(state.discover(&mut store), Err(Failure::Choice));
        state = State {
            chosen: Some(CredentialSource::Ide),
            ..State::default()
        };
        state.discover(&mut store).unwrap();
        assert_eq!(
            state.current.as_ref().unwrap().subject.as_deref(),
            Some("other")
        );
    }
    struct DualStore {
        ide: Credential,
        cli: Credential,
    }
    impl Store for DualStore {
        fn read(&mut self, source: CredentialSource) -> Result<Credential, Failure> {
            Ok(if source == CredentialSource::Ide {
                self.ide.clone()
            } else {
                self.cli.clone()
            })
        }
    }
    #[test]
    fn terminal_ide_session_falls_back_once_to_same_account_cli() {
        let mut cli = credential(9999);
        cli.source = CredentialSource::Cli;
        let mut store = DualStore {
            ide: credential(900),
            cli,
        };
        let mut state = State::default();
        let mut transport = FakeTransport {
            responses: VecDeque::from([
                response(200, json!({"shouldLogout":true})),
                response(200, json!({})),
            ]),
            calls: vec![],
        };
        let mut session = Session {
            state: &mut state,
            store: &mut store,
            transport: &mut transport,
            refreshed: false,
            reloaded: false,
            now: 1000,
        };
        session.begin().unwrap();
        session.post("GetPlanInfo", "{}").unwrap();
        session.finish().unwrap();
        assert!(session.refreshed);
        assert!(session.state.current.as_ref().unwrap().source == CredentialSource::Cli);
        assert_eq!(transport.calls, ["oauth/token", "GetPlanInfo"]);
    }
    #[test]
    fn fallback_does_not_override_explicit_source_or_other_account() {
        for explicit in [false, true] {
            let mut cli = credential(9999);
            cli.source = CredentialSource::Cli;
            if !explicit {
                cli.subject = Some("different".into());
            }
            let mut store = DualStore {
                ide: credential(900),
                cli,
            };
            let mut state = State {
                chosen: explicit.then_some(CredentialSource::Ide),
                ..State::default()
            };
            let mut transport = FakeTransport {
                responses: VecDeque::from([response(200, json!({"shouldLogout":true}))]),
                calls: vec![],
            };
            let mut session = Session {
                state: &mut state,
                store: &mut store,
                transport: &mut transport,
                refreshed: false,
                reloaded: false,
                now: 1000,
            };
            assert_eq!(
                session.begin(),
                Err(if explicit {
                    Failure::SignedOut
                } else {
                    Failure::Choice
                })
            );
            assert_eq!(transport.calls.len(), usize::from(explicit));
        }
    }
    #[test]
    fn transient_refresh_does_not_fallback_or_sign_out() {
        let mut cli = credential(9999);
        cli.source = CredentialSource::Cli;
        let mut store = DualStore {
            ide: credential(1100),
            cli,
        };
        let mut state = State::default();
        let mut transport = FakeTransport {
            responses: VecDeque::from([response(500, json!({})), response(200, json!({}))]),
            calls: vec![],
        };
        let mut session = Session {
            state: &mut state,
            store: &mut store,
            transport: &mut transport,
            refreshed: false,
            reloaded: false,
            now: 1000,
        };
        session.begin().unwrap();
        session.post("GetPlanInfo", "{}").unwrap();
        assert!(session.state.current.as_ref().unwrap().source == CredentialSource::Ide);
        assert!(session.state.rejected.is_none());
    }
    #[test]
    fn logout_after_successful_session_invalidates_account_epoch() {
        struct LoggedOut;
        impl Store for LoggedOut {
            fn read(&mut self, _: CredentialSource) -> Result<Credential, Failure> {
                Err(Failure::Missing)
            }
        }
        let mut state = State {
            current: Some(credential(9999)),
            ..State::default()
        };
        let before = cache_epoch();
        assert_eq!(state.discover(&mut LoggedOut), Err(Failure::Missing));
        assert!(state.current.is_none());
        assert!(cache_epoch() > before);
    }
    #[test]
    fn source_switch_blocks_suspended_old_publication_on_every_channel() {
        use std::sync::{Arc, Barrier, Mutex};
        let ready = Arc::new(Barrier::new(2));
        let resume = Arc::new(Barrier::new(2));
        let channels = Arc::new(Mutex::new(Vec::new()));
        let captured = cache_epoch();
        let worker = {
            let ready = ready.clone();
            let resume = resume.clone();
            let channels = channels.clone();
            std::thread::spawn(move || {
                ready.wait();
                resume.wait();
                publish_if_current(captured, || {
                    channels.lock().unwrap().extend([
                        "old cache",
                        "old event",
                        "old notification",
                        "old direct response",
                    ]);
                })
            })
        };
        ready.wait();
        select_source_with("cli", || {
            channels.lock().unwrap().push("new account selected")
        })
        .unwrap();
        resume.wait();
        assert!(worker.join().unwrap().is_none());
        assert_eq!(*channels.lock().unwrap(), ["new account selected"]);
        assert!(publish_if_current(captured, || "old delayed response").is_none());
    }
    #[test]
    fn previously_rejected_ide_does_not_hide_new_same_account_cli() {
        let ide = credential(900);
        let mut cli = credential(9999);
        cli.source = CredentialSource::Cli;
        let mut state = State {
            rejected: Some(ide.access.clone()),
            ..State::default()
        };
        let mut store = DualStore { ide, cli };
        let mut transport = FakeTransport {
            responses: VecDeque::from([response(200, json!({}))]),
            calls: vec![],
        };
        let mut session = Session {
            state: &mut state,
            store: &mut store,
            transport: &mut transport,
            refreshed: false,
            reloaded: false,
            now: 1000,
        };
        session.begin().unwrap();
        session.post("GetPlanInfo", "{}").unwrap();
        assert_eq!(transport.calls, ["GetPlanInfo"]);
    }
    #[test]
    fn protected_ide_allows_only_eligible_cli_fallback() {
        struct ProtectedIde(Credential);
        impl Store for ProtectedIde {
            fn read(&mut self, source: CredentialSource) -> Result<Credential, Failure> {
                match source {
                    CredentialSource::Ide => Err(Failure::Protected),
                    CredentialSource::Cli => Ok(self.0.clone()),
                }
            }
        }
        let mut cli = credential(9999);
        cli.source = CredentialSource::Cli;
        let mut store = ProtectedIde(cli);
        let mut initial = State::default();
        initial.discover(&mut store).unwrap();
        assert!(initial.current.unwrap().source == CredentialSource::Cli);
        let mut explicit = State {
            chosen: Some(CredentialSource::Ide),
            ..State::default()
        };
        assert_eq!(explicit.discover(&mut store), Err(Failure::Protected));
        store.0.subject = Some("another-account".into());
        let mut existing = State {
            current: Some(credential(9999)),
            ..State::default()
        };
        assert_eq!(existing.discover(&mut store), Err(Failure::Protected));
        assert!(existing.current.is_none());
    }
}
