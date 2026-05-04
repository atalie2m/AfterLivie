import SwiftUI

@main
struct AfterLivieApp: App {
    @StateObject private var appModel = AppModel()

    var body: some Scene {
        DocumentGroup(newDocument: ReplayDocument()) { file in
            ContentView(document: file.$document)
                .environmentObject(appModel)
                .frame(minWidth: 1080, minHeight: 680)
        }
        .commands {
            CommandGroup(after: .newItem) {
                Button("Run Doctor") {
                    appModel.runDoctor()
                }
                .keyboardShortcut("d", modifiers: [.command, .shift])
            }
        }

        Settings {
            SettingsView()
                .environmentObject(appModel)
        }
    }
}
