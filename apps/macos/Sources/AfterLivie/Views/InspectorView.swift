import SwiftUI

struct InspectorView: View {
    var project: ReplayProjectSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Inspector")
                .font(.headline)

            LabeledContent("Schema", value: project.schemaVersion)
            LabeledContent("Offset", value: "\(project.globalOffsetMs) ms")
            LabeledContent("Diagnostics", value: "\(project.diagnostics.count)")
            LabeledContent("Renders", value: "\(project.renderHistory.count)")

            Spacer()
        }
        .padding(18)
        .background(.thinMaterial)
    }
}
