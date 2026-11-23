import SwiftUI

struct ContentView: View {
    @Binding var document: ReplayDocument
    @State private var selection: SidebarSection? = .project

    var body: some View {
        NavigationSplitView {
            SidebarView(selection: $selection, project: document.project)
        } detail: {
            HSplitView {
                DetailView(document: $document, selection: selection ?? .project)
                    .frame(minWidth: 620)

                InspectorView(project: $document.project)
                    .frame(minWidth: 280, idealWidth: 320, maxWidth: 380)
            }
        }
    }
}

enum SidebarSection: String, CaseIterable, Identifiable {
    case project
    case importMedia
    case sync
    case preview
    case export
    case diagnostics

    var id: String { rawValue }

    var title: String {
        switch self {
        case .project: "Project"
        case .importMedia: "Import"
        case .sync: "Sync"
        case .preview: "Preview"
        case .export: "Export"
        case .diagnostics: "Diagnostics"
        }
    }

    var systemImage: String {
        switch self {
        case .project: "doc"
        case .importMedia: "square.and.arrow.down"
        case .sync: "arrow.left.arrow.right"
        case .preview: "play.rectangle"
        case .export: "square.and.arrow.up"
        case .diagnostics: "exclamationmark.triangle"
        }
    }
}
