import SwiftUI

struct RenderActionView: View {
    @EnvironmentObject private var appModel: AppModel
    var title: String
    var actionTitle: String
    var action: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text(title)
                .font(.headline)
            Button(actionTitle, action: action)
                .disabled(appModel.isRunningCommand)

            if !appModel.lastCommandOutput.isEmpty {
                Text(appModel.lastCommandOutput)
                    .font(.system(.caption, design: .monospaced))
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }
}
