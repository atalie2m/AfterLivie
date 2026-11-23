import SwiftUI

struct SettingsView: View {
    @EnvironmentObject private var appModel: AppModel

    var body: some View {
        Form {
            TextField("CLI Path", text: $appModel.cliPath)
            LabeledContent("Renderer", value: "rust-cpu-overlay")
            LabeledContent("Font Policy", value: "system-default-v1")
            LabeledContent("Default Preset", value: "high_quality_upload")
        }
        .padding()
        .frame(width: 520)
    }
}
