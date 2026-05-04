import Foundation

@MainActor
final class AppModel: ObservableObject {
    @Published var diagnostics: [DiagnosticItem] = []
    @Published var isRunningCommand = false
    @Published var lastCommandOutput = ""
    @Published var cliPath = ProcessInfo.processInfo.environment["AFTERLIVIE_CLI"] ?? "afterlivie"

    private let service = ReplayCLIService()

    func runDoctor() {
        run(arguments: ["doctor"])
    }

    func probe(videoPath: String) {
        run(arguments: ["probe-media", "--video", videoPath])
    }

    func preview(videoPath: String, commentsPath: String, outputPath: String) {
        run(arguments: [
            "preview-export",
            "--video", videoPath,
            "--comments", commentsPath,
            "--out", outputPath
        ])
    }

    func export(videoPath: String, commentsPath: String, outputPath: String) {
        run(arguments: [
            "full-export",
            "--video", videoPath,
            "--comments", commentsPath,
            "--out", outputPath
        ])
    }

    private func run(arguments: [String]) {
        isRunningCommand = true
        lastCommandOutput = ""

        Task {
            let result = await service.run(cliPath: cliPath, arguments: arguments)
            diagnostics = result.diagnostics
            lastCommandOutput = result.output
            isRunningCommand = false
        }
    }
}
