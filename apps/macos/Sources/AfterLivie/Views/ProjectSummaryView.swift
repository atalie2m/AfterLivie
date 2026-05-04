import SwiftUI

struct ProjectSummaryView: View {
    var project: ReplayProjectSummary

    var body: some View {
        Grid(alignment: .leading, horizontalSpacing: 18, verticalSpacing: 12) {
            GridRow {
                Text("Project ID").foregroundStyle(.secondary)
                Text(project.projectId).textSelection(.enabled)
            }
            GridRow {
                Text("Video").foregroundStyle(.secondary)
                Text(project.sourceVideoPath ?? "Not selected").textSelection(.enabled)
            }
            GridRow {
                Text("Comment Sources").foregroundStyle(.secondary)
                Text(project.commentSources.isEmpty ? "Not selected" : "\(project.commentSources.count)")
            }
            GridRow {
                Text("Layout").foregroundStyle(.secondary)
                Text(project.layoutTemplateId)
            }
        }

        if !project.commentSources.isEmpty {
            VStack(alignment: .leading, spacing: 8) {
                Text("Sources")
                    .font(.headline)
                ForEach(project.commentSources) { source in
                    HStack {
                        Text(source.displayName)
                        Text(source.format)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Spacer()
                        Text(source.enabled ? "Enabled" : "Disabled")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Text(source.path)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                }
            }
            .padding(.top, 18)
        }
    }
}
