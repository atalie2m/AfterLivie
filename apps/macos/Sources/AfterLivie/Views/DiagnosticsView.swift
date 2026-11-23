import SwiftUI

struct DiagnosticsView: View {
    @Binding var project: ReplayProjectSummary
    var diagnostics: [DiagnosticItem]
    var refreshTimeline: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            HStack {
                Text("Merged Timeline")
                    .font(.headline)
                Spacer()
                Button(action: refreshTimeline) {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .disabled(project.commentSources.isEmpty)
            }

            if !project.mergedTimelineSources.isEmpty {
                Grid(alignment: .leading, horizontalSpacing: 14, verticalSpacing: 6) {
                    GridRow {
                        Text("Source").font(.caption).foregroundStyle(.secondary)
                        Text("Events").font(.caption).foregroundStyle(.secondary)
                        Text("Skipped").font(.caption).foregroundStyle(.secondary)
                        Text("Offset").font(.caption).foregroundStyle(.secondary)
                        Text("Priority").font(.caption).foregroundStyle(.secondary)
                        Text("State").font(.caption).foregroundStyle(.secondary)
                    }
                    ForEach(project.mergedTimelineSources) { source in
                        GridRow {
                            Text(source.displayName)
                            Text("\(source.commentCount)")
                            Text("\(source.skippedCount)")
                            Text("\(source.offsetMs) ms")
                            Text("\(source.priority)")
                            Text(source.enabled ? "Enabled" : "Disabled")
                        }
                    }
                }
            }

            if !project.alignmentSuggestions.isEmpty {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Alignment Suggestions")
                        .font(.subheadline)
                        .fontWeight(.semibold)
                    ForEach(project.alignmentSuggestions) { suggestion in
                        HStack(alignment: .firstTextBaseline) {
                            Text(suggestion.sourceId)
                            Text("\(suggestion.suggestedOffsetMs) ms")
                                .font(.caption.monospacedDigit())
                            Text("\(Int(suggestion.confidence * 100))%")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                            Text(suggestion.basis)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }

            if !project.mergedTimelineRows.isEmpty {
                Grid(alignment: .leading, horizontalSpacing: 14, verticalSpacing: 6) {
                    GridRow {
                        Text("Effective").font(.caption).foregroundStyle(.secondary)
                        Text("Original").font(.caption).foregroundStyle(.secondary)
                        Text("Source").font(.caption).foregroundStyle(.secondary)
                        Text("Platform").font(.caption).foregroundStyle(.secondary)
                        Text("Author").font(.caption).foregroundStyle(.secondary)
                        Text("Text").font(.caption).foregroundStyle(.secondary)
                    }
                    ForEach(Array(project.mergedTimelineRows.prefix(100))) { row in
                        GridRow {
                            Text("\(row.effectiveTimestampMs) ms")
                                .font(.caption.monospacedDigit())
                            Text("\(row.originalTimestampMs) ms")
                                .font(.caption.monospacedDigit())
                            Text(row.sourceDisplayName)
                            Text(row.platformLabel ?? row.platform ?? "-")
                            Text(row.authorDisplayName ?? "-")
                            Text(row.text).lineLimit(2)
                        }
                    }
                }
            }

            Divider()

            if diagnostics.isEmpty {
                ContentUnavailableView("No Diagnostics", systemImage: "checkmark.circle")
            } else {
                VStack(alignment: .leading, spacing: 12) {
                    ForEach(diagnostics) { diagnostic in
                        DiagnosticCard(diagnostic: diagnostic)
                    }
                }
            }
        }
    }
}

private struct DiagnosticCard: View {
    var diagnostic: DiagnosticItem

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(diagnostic.severity.uppercased())
                    .font(.caption)
                    .fontWeight(.semibold)
                Text(diagnostic.category)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
                Text(diagnostic.code)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
            }
            Text(diagnostic.message)
            if let hint = diagnostic.actionableHint {
                Text(hint)
                    .font(.callout)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(12)
        .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 8))
    }
}
