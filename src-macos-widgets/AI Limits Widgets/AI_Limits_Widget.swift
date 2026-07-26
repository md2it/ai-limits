import SwiftUI
import WidgetKit

struct AILimitsWidgetEntry: TimelineEntry {
    let date: Date
}

struct AILimitsWidgetProvider: TimelineProvider {
    func placeholder(in context: Context) -> AILimitsWidgetEntry {
        AILimitsWidgetEntry(date: Date())
    }

    func getSnapshot(in context: Context, completion: @escaping (AILimitsWidgetEntry) -> Void) {
        completion(AILimitsWidgetEntry(date: Date()))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<AILimitsWidgetEntry>) -> Void) {
        let entry = AILimitsWidgetEntry(date: Date())
        completion(Timeline(entries: [entry], policy: .after(Date().addingTimeInterval(15 * 60))))
    }
}

struct AILimitsWidgetView: View {
    var entry: AILimitsWidgetProvider.Entry

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("AI Limits")
                .font(.headline)
            Text("No snapshot")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .padding()
    }
}

struct AILimitsWidget: Widget {
    static let kind = "AILimitsWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: Self.kind, provider: AILimitsWidgetProvider()) { entry in
            AILimitsWidgetView(entry: entry)
        }
        .configurationDisplayName("AI Limits")
        .description("Shows the latest saved provider limits.")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
    }
}
