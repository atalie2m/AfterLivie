import AppKit
import SwiftUI

struct ImportView: View {
    @Binding var document: ReplayDocument
    @EnvironmentObject private var appModel: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            FileSelectionRow(
                title: "Source Video",
                path: document.project.sourceVideoPath,
                buttonTitle: "Choose Video",
                action: chooseVideo
            )

            FileSelectionRow(
                title: "Canonical JSON",
                path: document.project.commentSourcePath,
                buttonTitle: "Choose Comments",
                action: chooseComments
            )

            Button("Probe Video") {
                if let path = document.project.sourceVideoPath {
                    appModel.probe(videoPath: path)
                }
            }
            .disabled(document.project.sourceVideoPath == nil)
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
        panel.allowedContentTypes = [.json]
        panel.allowsMultipleSelection = false
        if panel.runModal() == .OK {
            document.project.commentSourcePath = panel.url?.path
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
