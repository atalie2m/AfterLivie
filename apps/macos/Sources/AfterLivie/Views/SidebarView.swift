import SwiftUI

struct SidebarView: View {
    @Binding var selection: SidebarSection?
    var project: ReplayProjectSummary

    var body: some View {
        List(selection: $selection) {
            Section("Workflow") {
                ForEach(SidebarSection.allCases) { section in
                    Label(section.title, systemImage: section.systemImage)
                        .tag(section)
                }
            }

            Section("Project") {
                LabeledContent("Layout", value: project.layoutTemplateId)
                LabeledContent("Offset", value: "\(project.globalOffsetMs) ms")
                LabeledContent("Sources", value: "\(project.commentSources.count)")
            }
        }
        .listStyle(.sidebar)
        .navigationTitle("AfterLivie")
    }
}
