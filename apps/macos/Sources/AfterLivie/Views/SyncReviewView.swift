import SwiftUI

struct SyncReviewView: View {
    @Binding var document: ReplayDocument
    @EnvironmentObject private var appModel: AppModel
    @State private var selectedSourceId: String?
    @State private var targetAnchorRowId: String?
    @State private var referenceAnchorRowId: String?

    private let layoutTemplates = [
        ("classic-sidebar-v1", "Classic Sidebar"),
        ("merged-multiplatform-v1", "Merged Multi-Platform"),
        ("split-platform-review-v1", "Split Platform Review")
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            header

            if document.project.commentSources.isEmpty {
                ContentUnavailableView("No Comment Sources", systemImage: "text.bubble")
            } else {
                layoutAndPriority
                offsetControls
                alignmentSuggestions
                sourcePreviews
            }
        }
        .onAppear {
            if selectedSourceId == nil {
                selectedSourceId = document.project.commentSources.first?.sourceId
            }
            if document.project.mergedTimelineRows.isEmpty {
                refreshMergedTimeline()
            }
        }
    }

    private var header: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text("Multi-Source Sync")
                    .font(.headline)
                Text("\(document.project.commentSources.count) sources")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button(action: refreshMergedTimeline) {
                Label("Refresh", systemImage: "arrow.clockwise")
            }
            .disabled(document.project.commentSources.isEmpty)
        }
    }

    private var layoutAndPriority: some View {
        VStack(alignment: .leading, spacing: 12) {
            Picker("Render Layout", selection: $document.project.layoutTemplateId) {
                ForEach(layoutTemplates, id: \.0) { template in
                    Text(template.1).tag(template.0)
                }
            }
            .pickerStyle(.menu)

            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 8) {
                GridRow {
                    Text("Priority").font(.caption).foregroundStyle(.secondary)
                    Text("Source").font(.caption).foregroundStyle(.secondary)
                    Text("Platform").font(.caption).foregroundStyle(.secondary)
                    Text("Events").font(.caption).foregroundStyle(.secondary)
                    Text("UTC Start").font(.caption).foregroundStyle(.secondary)
                    Text("").font(.caption)
                }
                ForEach(Array(document.project.commentSources.enumerated()), id: \.element.sourceId) { index, source in
                    GridRow {
                        Text("\(index)")
                            .font(.caption.monospacedDigit())
                        TextField("Display Name", text: displayNameBinding(for: source.sourceId))
                            .textFieldStyle(.roundedBorder)
                        TextField("Platform", text: platformBinding(for: source.sourceId))
                            .textFieldStyle(.roundedBorder)
                        Text("\(source.commentCount)")
                            .font(.caption.monospacedDigit())
                        TextField("syncStartUtc", text: metadataBinding(for: source.sourceId, key: "syncStartUtc"))
                            .textFieldStyle(.roundedBorder)
                        HStack(spacing: 6) {
                            Button(action: { moveSource(source.sourceId, by: -1) }) {
                                Image(systemName: "arrow.up")
                            }
                            .disabled(index == 0)
                            Button(action: { moveSource(source.sourceId, by: 1) }) {
                                Image(systemName: "arrow.down")
                            }
                            .disabled(index == document.project.commentSources.count - 1)
                        }
                        .buttonStyle(.borderless)
                    }
                }
            }
        }
    }

    private var offsetControls: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Picker("Selected Source", selection: selectedSourceBinding) {
                    ForEach(document.project.commentSources) { source in
                        Text(source.displayName).tag(source.sourceId)
                    }
                }
                .pickerStyle(.menu)

                Text("Global \(document.project.globalOffsetMs) ms")
                    .foregroundStyle(.secondary)

                Spacer()
            }

            HStack(spacing: 8) {
                Button("-1000 ms") { nudgeSelected(by: -1000) }
                    .keyboardShortcut(.leftArrow, modifiers: [.command, .shift])
                Button("-100 ms") { nudgeSelected(by: -100) }
                    .keyboardShortcut(.leftArrow, modifiers: [.command, .option])
                Text(selectedOffsetLabel)
                    .font(.callout.monospacedDigit())
                    .frame(minWidth: 120)
                Button("+100 ms") { nudgeSelected(by: 100) }
                    .keyboardShortcut(.rightArrow, modifiers: [.command, .option])
                Button("+1000 ms") { nudgeSelected(by: 1000) }
                    .keyboardShortcut(.rightArrow, modifiers: [.command, .shift])
                Spacer()
                Button(action: applyAnchorSync) {
                    Label("Apply Anchor Sync", systemImage: "link")
                }
                .disabled(!canApplyAnchorSync)
            }
        }
    }

    @ViewBuilder
    private var alignmentSuggestions: some View {
        if !document.project.alignmentSuggestions.isEmpty {
            VStack(alignment: .leading, spacing: 10) {
                Text("Automatic Alignment Suggestions")
                    .font(.headline)
                ForEach(document.project.alignmentSuggestions) { suggestion in
                    HStack(alignment: .firstTextBaseline) {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(sourceName(for: suggestion.sourceId))
                                .font(.subheadline)
                                .fontWeight(.semibold)
                            Text("\(suggestion.suggestedOffsetMs) ms - confidence \(Int(suggestion.confidence * 100))%")
                                .font(.caption.monospacedDigit())
                                .foregroundStyle(.secondary)
                            Text(suggestion.reason)
                                .font(.callout)
                                .foregroundStyle(.secondary)
                                .lineLimit(2)
                        }
                        Spacer()
                        Button("Apply Suggested Offset") {
                            applySuggestion(suggestion)
                        }
                    }
                }
            }
        }
    }

    private var sourcePreviews: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text("Side-by-Side Source Preview")
                    .font(.headline)
                Spacer()
                if let target = targetAnchorRowId, let reference = referenceAnchorRowId {
                    Text("Target \(target) / Reference \(reference)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }

            ScrollView(.horizontal) {
                HStack(alignment: .top, spacing: 12) {
                    ForEach(document.project.commentSources) { source in
                        SourcePreviewColumn(
                            source: source,
                            priority: priority(for: source.sourceId),
                            rows: rows(for: source.sourceId),
                            targetAnchorRowId: targetAnchorRowId,
                            referenceAnchorRowId: referenceAnchorRowId,
                            chooseTarget: { targetAnchorRowId = $0 },
                            chooseReference: { referenceAnchorRowId = $0 }
                        )
                        .frame(width: 360, alignment: .topLeading)
                    }
                }
            }
        }
    }

    private var selectedSourceBinding: Binding<String> {
        Binding(
            get: { selectedSourceId ?? document.project.commentSources.first?.sourceId ?? "" },
            set: { selectedSourceId = $0 }
        )
    }

    private var selectedOffsetLabel: String {
        guard let sourceId = selectedSourceId,
              let source = document.project.commentSources.first(where: { $0.sourceId == sourceId }) else {
            return "No source"
        }
        return "\(source.offsetMs) ms"
    }

    private var canApplyAnchorSync: Bool {
        guard let targetAnchorRowId,
              let referenceAnchorRowId,
              let target = document.project.mergedTimelineRows.first(where: { $0.id == targetAnchorRowId }),
              let reference = document.project.mergedTimelineRows.first(where: { $0.id == referenceAnchorRowId }) else {
            return false
        }
        return target.sourceId != reference.sourceId
    }

    private func rows(for sourceId: String) -> [MergedTimelineRowSummary] {
        document.project.mergedTimelineRows
            .filter { $0.sourceId == sourceId }
            .prefix(30)
            .map { $0 }
    }

    private func priority(for sourceId: String) -> Int {
        document.project.mergedTimelineSources.first(where: { $0.sourceId == sourceId })?.priority
            ?? document.project.commentSources.firstIndex(where: { $0.sourceId == sourceId })
            ?? 0
    }

    private func sourceName(for sourceId: String) -> String {
        document.project.commentSources.first(where: { $0.sourceId == sourceId })?.displayName ?? sourceId
    }

    private func displayNameBinding(for sourceId: String) -> Binding<String> {
        Binding(
            get: { document.project.commentSources.first(where: { $0.sourceId == sourceId })?.displayName ?? "" },
            set: { value in
                if let index = document.project.commentSources.firstIndex(where: { $0.sourceId == sourceId }) {
                    document.project.commentSources[index].displayName = value
                }
            }
        )
    }

    private func platformBinding(for sourceId: String) -> Binding<String> {
        Binding(
            get: { document.project.commentSources.first(where: { $0.sourceId == sourceId })?.platform ?? "" },
            set: { value in
                if let index = document.project.commentSources.firstIndex(where: { $0.sourceId == sourceId }) {
                    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
                    document.project.commentSources[index].platform = trimmed.isEmpty ? nil : trimmed
                }
            }
        )
    }

    private func metadataBinding(for sourceId: String, key: String) -> Binding<String> {
        Binding(
            get: { document.project.commentSources.first(where: { $0.sourceId == sourceId })?.metadata[key] ?? "" },
            set: { value in
                if let index = document.project.commentSources.firstIndex(where: { $0.sourceId == sourceId }) {
                    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
                    if trimmed.isEmpty {
                        document.project.commentSources[index].metadata.removeValue(forKey: key)
                    } else {
                        document.project.commentSources[index].metadata[key] = trimmed
                    }
                }
            }
        )
    }

    private func nudgeSelected(by delta: Int) {
        guard let sourceId = selectedSourceId,
              let index = document.project.commentSources.firstIndex(where: { $0.sourceId == sourceId }) else {
            return
        }
        document.project.commentSources[index].offsetMs += delta
        refreshMergedTimeline()
    }

    private func moveSource(_ sourceId: String, by delta: Int) {
        guard let index = document.project.commentSources.firstIndex(where: { $0.sourceId == sourceId }) else {
            return
        }
        let destination = index + delta
        guard document.project.commentSources.indices.contains(destination) else {
            return
        }
        document.project.commentSources.swapAt(index, destination)
        refreshMergedTimeline()
    }

    private func applyAnchorSync() {
        guard let targetAnchorRowId,
              let referenceAnchorRowId,
              let target = document.project.mergedTimelineRows.first(where: { $0.id == targetAnchorRowId }),
              let reference = document.project.mergedTimelineRows.first(where: { $0.id == referenceAnchorRowId }),
              target.sourceId != reference.sourceId,
              let index = document.project.commentSources.firstIndex(where: { $0.sourceId == target.sourceId }) else {
            return
        }
        document.project.commentSources[index].offsetMs += reference.effectiveTimestampMs - target.effectiveTimestampMs
        selectedSourceId = target.sourceId
        refreshMergedTimeline()
    }

    private func applySuggestion(_ suggestion: AlignmentSuggestionSummary) {
        guard let index = document.project.commentSources.firstIndex(where: { $0.sourceId == suggestion.sourceId }) else {
            return
        }
        document.project.commentSources[index].offsetMs = suggestion.suggestedOffsetMs
        selectedSourceId = suggestion.sourceId
        refreshMergedTimeline()
    }

    private func refreshMergedTimeline() {
        Task {
            if let report = await appModel.mergedTimeline(
                sources: document.project.commentSources,
                globalOffsetMs: document.project.globalOffsetMs
            ) {
                document.project.mergedTimelineRows = report.rows
                document.project.mergedTimelineSources = report.sourceSummaries
                document.project.alignmentSuggestions = report.alignmentSuggestions
                document.project.orderingRules = report.orderingRules
                document.project.diagnostics = report.diagnostics
                for summary in report.sourceSummaries {
                    if let index = document.project.commentSources.firstIndex(where: { $0.sourceId == summary.sourceId }) {
                        document.project.commentSources[index].commentCount = summary.commentCount
                        document.project.commentSources[index].skippedCount = Int(summary.skippedCount)
                    }
                }
            }
        }
    }
}

private struct SourcePreviewColumn: View {
    var source: CommentSourceSummary
    var priority: Int
    var rows: [MergedTimelineRowSummary]
    var targetAnchorRowId: String?
    var referenceAnchorRowId: String?
    var chooseTarget: (String) -> Void
    var chooseReference: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(source.displayName)
                        .font(.subheadline)
                        .fontWeight(.semibold)
                    Spacer()
                    Text("#\(priority)")
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(.secondary)
                }
                Text("\(source.platform ?? "No platform") - offset \(source.offsetMs) ms")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            if rows.isEmpty {
                ContentUnavailableView("No Rows", systemImage: "timeline.selection")
                    .frame(minHeight: 120)
            } else {
                Grid(alignment: .leading, horizontalSpacing: 8, verticalSpacing: 6) {
                    GridRow {
                        Text("Use").font(.caption).foregroundStyle(.secondary)
                        Text("Original").font(.caption).foregroundStyle(.secondary)
                        Text("Effective").font(.caption).foregroundStyle(.secondary)
                        Text("Text").font(.caption).foregroundStyle(.secondary)
                    }
                    ForEach(rows) { row in
                        GridRow {
                            HStack(spacing: 4) {
                                Button("T") { chooseTarget(row.id) }
                                    .buttonStyle(.borderless)
                                    .fontWeight(targetAnchorRowId == row.id ? .bold : .regular)
                                Button("R") { chooseReference(row.id) }
                                    .buttonStyle(.borderless)
                                    .fontWeight(referenceAnchorRowId == row.id ? .bold : .regular)
                            }
                            Text("\(row.originalTimestampMs)")
                                .font(.caption.monospacedDigit())
                            Text("\(row.effectiveTimestampMs)")
                                .font(.caption.monospacedDigit())
                            VStack(alignment: .leading, spacing: 2) {
                                Text(row.authorDisplayName ?? "-")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                Text(row.text)
                                    .lineLimit(2)
                            }
                        }
                    }
                }
            }
        }
        .textSelection(.enabled)
    }
}
