import SwiftUI

struct InspectorView: View {
    @Binding var project: ReplayProjectSummary

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text("Inspector")
                    .font(.headline)

                LabeledContent("Schema", value: project.schemaVersion)
                LabeledContent("Global Offset", value: "\(project.globalOffsetMs) ms")
                LabeledContent("Diagnostics", value: "\(project.diagnostics.count)")
                LabeledContent("Renders", value: "\(project.renderHistory.count)")
                Picker("Layout", selection: $project.layoutTemplateId) {
                    Text("Classic Sidebar").tag("classic-sidebar-v1")
                    Text("Merged Multi-Platform").tag("merged-multiplatform-v1")
                    Text("Split Platform Review").tag("split-platform-review-v1")
                }

                Divider()

                Text("Sources")
                    .font(.headline)

                if project.commentSources.isEmpty {
                    Text("No comment sources")
                        .foregroundStyle(.secondary)
                } else {
                    ForEach($project.commentSources) { $source in
                        SourceInspectorSection(source: $source)
                        Divider()
                    }
                }
            }
            .padding(18)
        }
        .background(.thinMaterial)
    }
}

private struct SourceInspectorSection: View {
    @Binding var source: CommentSourceSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            TextField("Display Name", text: $source.displayName)
                .font(.headline)

            Toggle("Enabled", isOn: $source.enabled)

            TextField("Platform", text: Binding(
                get: { source.platform ?? "" },
                set: { source.platform = $0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? nil : $0 }
            ))

            TextField("Offset (ms)", value: $source.offsetMs, format: .number)
            TextField("Sync Start UTC", text: Binding(
                get: { source.metadata["syncStartUtc"] ?? "" },
                set: { value in
                    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
                    if trimmed.isEmpty {
                        source.metadata.removeValue(forKey: "syncStartUtc")
                    } else {
                        source.metadata["syncStartUtc"] = trimmed
                    }
                }
            ))

            LabeledContent("Source ID", value: source.sourceId)
            LabeledContent("Format", value: source.format)
            LabeledContent("Importer", value: importerLabel)
            LabeledContent("File", value: source.path)
            LabeledContent("Timestamp Basis", value: source.timestampBasis)
            LabeledContent("Events", value: "\(source.commentCount)")
            LabeledContent("Skipped", value: "\(source.skippedCount)")

            if !source.metadata.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Metadata")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    ForEach(source.metadata.sorted(by: { $0.key < $1.key }), id: \.key) { key, value in
                        LabeledContent(key, value: value)
                    }
                }
            }

            if !source.diagnostics.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Import Diagnostics")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    ForEach(source.diagnostics.prefix(4)) { diagnostic in
                        Text("\(diagnostic.code): \(diagnostic.message)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(3)
                    }
                }
            }
        }
        .textSelection(.enabled)
    }

    private var importerLabel: String {
        [source.importerId, source.importerVersion]
            .compactMap { $0 }
            .joined(separator: " ")
            .isEmpty ? "Not previewed" : [source.importerId, source.importerVersion].compactMap { $0 }.joined(separator: " ")
    }
}
