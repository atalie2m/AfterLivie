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
                Text("Comments").foregroundStyle(.secondary)
                Text(project.commentSourcePath ?? "Not selected").textSelection(.enabled)
            }
            GridRow {
                Text("Layout").foregroundStyle(.secondary)
                Text(project.layoutTemplateId)
            }
        }
    }
}
