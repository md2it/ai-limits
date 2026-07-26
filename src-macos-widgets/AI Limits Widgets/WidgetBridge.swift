import Foundation
import WidgetKit

enum AILimitsWidgetBridge {
    static let appGroupIdentifier = "group.md2it.ai-limits.shared"

    static func sharedContainerURL() -> URL? {
        FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroupIdentifier)
    }

    static func reloadAllTimelines() {
        WidgetCenter.shared.reloadTimelines(ofKind: AILimitsWidget.kind)
    }
}
