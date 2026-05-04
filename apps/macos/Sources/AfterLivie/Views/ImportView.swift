import AppKit
import SwiftUI

struct ImportView: View {
    @Binding var document: ReplayDocument
    @EnvironmentObject private var appModel: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            FileSelectionRow(
                title: "Source Video",
                path: document.project.sourceVideoPath,
                buttonTitle: "Choose Video",
                action: chooseVideo
            )

            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Comment Sources")
                        .font(.headline)
                    Text("\(document.project.commentSources.count) selected")
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Button(action: chooseComments) {
                    Label("Add Sources", systemImage: "plus")
                }
            }

            if document.project.commentSources.isEmpty {
                ContentUnavailableView("No Comment Sources", systemImage: "text.bubble")
                    .frame(maxWidth: .infinity, minHeight: 180)
            } else {
                VStack(alignment: .leading, spacing: 14) {
                    ForEach($document.project.commentSources) { $source in
                        SourceImportSection(
                            source: $source,
                            refreshPreview: { refreshPreview(sourceID: source.sourceId) },
                            remove: { removeSource(sourceID: source.sourceId) }
                        )
                    }
                }
            }

            HStack {
                Button(action: probeVideo) {
                    Label("Probe Video", systemImage: "waveform.path.ecg")
                }
                .disabled(document.project.sourceVideoPath == nil)

                Button(action: refreshMergedTimeline) {
                    Label("Refresh Timeline", systemImage: "timeline.selection")
                }
                .disabled(document.project.commentSources.isEmpty)
            }
        }
    }

    private func chooseVideo() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.movie, .mpeg4Movie, .quickTimeMovie]
        panel.allowsMultipleSelection = false
        if panel.runModal() == .OK {
            document.project.sourceVideoPath = panel.url?.path
            if let path = document.project.sourceVideoPath {
                appModel.probe(videoPath: path)
            }
        }
    }

    private func chooseComments() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json, .jsonLines, .commaSeparatedValues]
        panel.allowsMultipleSelection = true
        if panel.runModal() == .OK {
            for url in panel.urls {
                addSource(path: url.path)
            }
        }
    }

    private func addSource(path: String) {
        let base = CommentSourceSummary(path: path)
        var source = base
        let existingIDs = Set(document.project.commentSources.map(\.sourceId))
        if existingIDs.contains(source.sourceId) {
            var suffix = 2
            let root = source.sourceId
            while existingIDs.contains("\(root)_\(suffix)") {
                suffix += 1
            }
            source.sourceId = "\(root)_\(suffix)"
        }
        document.project.commentSources.append(source)
        refreshPreview(sourceID: source.sourceId)
    }

    private func removeSource(sourceID: String) {
        document.project.commentSources.removeAll { $0.sourceId == sourceID }
        document.project.mergedTimelineRows = []
        document.project.mergedTimelineSources = []
    }

    private func refreshPreview(sourceID: String) {
        guard let source = document.project.commentSources.first(where: { $0.sourceId == sourceID }) else {
            return
        }
        Task {
            if let preview = await appModel.importPreview(source: source),
               let index = document.project.commentSources.firstIndex(where: { $0.sourceId == sourceID }) {
                document.project.commentSources[index].apply(preview: preview)
                document.project.diagnostics = preview.diagnostics
            }
        }
    }

    private func refreshMergedTimeline() {
        Task {
            if let report = await appModel.mergedTimeline(
                sources: document.project.commentSources,
                globalOffsetMs: document.project.globalOffsetMs
            ) {
                document.project.mergedTimelineRows = report.rows
                document.project.mergedTimelineSources = report.sourceSummaries
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

    private func probeVideo() {
        if let path = document.project.sourceVideoPath {
            appModel.probe(videoPath: path)
        }
    }
}

private struct FileSelectionRow: View {
    var title: String
    var path: String?
    var buttonTitle: String
    var action: () -> Void

    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.headline)
                Text(path ?? "No file selected")
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
                    .textSelection(.enabled)
            }
            Spacer()
            Button(buttonTitle, action: action)
        }
    }
}

private struct SourceImportSection: View {
    @Binding var source: CommentSourceSummary
    var refreshPreview: () -> Void
    var remove: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .firstTextBaseline) {
                VStack(alignment: .leading, spacing: 3) {
                    Text(source.displayName)
                        .font(.headline)
                    Text(source.path)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)
                        .textSelection(.enabled)
                }
                Spacer()
                Button(action: refreshPreview) {
                    Label("Preview", systemImage: "tablecells")
                }
                Button(role: .destructive, action: remove) {
                    Label("Remove", systemImage: "minus.circle")
                }
            }

            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 8) {
                GridRow {
                    Text("Source ID").foregroundStyle(.secondary)
                    TextField("Source ID", text: $source.sourceId)
                        .textFieldStyle(.roundedBorder)
                }
                GridRow {
                    Text("Format").foregroundStyle(.secondary)
                    Text(source.format)
                }
                GridRow {
                    Text("Display").foregroundStyle(.secondary)
                    TextField("Display Name", text: $source.displayName)
                        .textFieldStyle(.roundedBorder)
                }
            }

            if source.format == "csv" {
                CsvMappingEditor(source: $source)
            }

            ImportPreviewTable(rows: source.previewRows)
        }
        .padding(12)
        .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 8))
    }
}

private struct CsvMappingEditor: View {
    @Binding var source: CommentSourceSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Column Mapping")
                .font(.subheadline)
                .fontWeight(.semibold)

            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 8) {
                mappingRow("Timestamp", binding: requiredBinding(\.timestamp))
                mappingRow("Text", binding: requiredBinding(\.text))
                mappingRow("ID", binding: optionalBinding(\.id))
                mappingRow("Author", binding: optionalBinding(\.author))
                mappingRow("Kind", binding: optionalBinding(\.kind))
                mappingRow("Platform", binding: optionalBinding(\.platform))
            }
        }
    }

    @ViewBuilder
    private func mappingRow(_ title: String, binding: Binding<String>) -> some View {
        GridRow {
            Text(title).foregroundStyle(.secondary)
            Picker(title, selection: binding) {
                Text("Unmapped").tag("")
                ForEach(source.headers, id: \.self) { header in
                    Text(header).tag(header)
                }
            }
            .labelsHidden()
            .pickerStyle(.menu)
            .frame(maxWidth: 220)
        }
    }

    private func requiredBinding(_ keyPath: WritableKeyPath<CsvColumnMappingSummary, String>) -> Binding<String> {
        Binding(
            get: { source.csvMapping?[keyPath: keyPath] ?? "" },
            set: { value in
                var mapping = source.csvMapping ?? CsvColumnMappingSummary()
                mapping[keyPath: keyPath] = value
                source.csvMapping = mapping
            }
        )
    }

    private func optionalBinding(_ keyPath: WritableKeyPath<CsvColumnMappingSummary, String?>) -> Binding<String> {
        Binding(
            get: { source.csvMapping?[keyPath: keyPath] ?? "" },
            set: { value in
                var mapping = source.csvMapping ?? CsvColumnMappingSummary()
                let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
                mapping[keyPath: keyPath] = trimmed.isEmpty ? nil : trimmed
                source.csvMapping = mapping
            }
        )
    }
}

private struct ImportPreviewTable: View {
    var rows: [ImportPreviewRowSummary]

    var body: some View {
        if rows.isEmpty {
            Text("No preview rows")
                .font(.callout)
                .foregroundStyle(.secondary)
        } else {
            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 6) {
                GridRow {
                    Text("Time").font(.caption).foregroundStyle(.secondary)
                    Text("Author").font(.caption).foregroundStyle(.secondary)
                    Text("Text").font(.caption).foregroundStyle(.secondary)
                }
                ForEach(rows.prefix(100)) { row in
                    GridRow {
                        Text("\(row.timestampMs) ms")
                            .font(.caption.monospacedDigit())
                        Text(row.authorDisplayName ?? "-")
                            .lineLimit(1)
                        Text(row.text)
                            .lineLimit(2)
                    }
                    .font(.callout)
                }
            }
        }
    }
}
