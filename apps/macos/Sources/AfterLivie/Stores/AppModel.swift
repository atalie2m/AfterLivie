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

    func importPreview(source: CommentSourceSummary) async -> ImportPreviewSummary? {
        do {
            let specPath = try writeSourceSpec(source)
            let result = await runForResult(arguments: [
                "import-preview",
                "--source-spec", specPath
            ])
            return decode(ImportPreviewSummary.self, from: result.output)
        } catch {
            diagnostics = [uiDiagnostic(code: "ui.import_preview_failed", message: error.localizedDescription)]
            return nil
        }
    }

    func mergedTimeline(
        sources: [CommentSourceSummary],
        globalOffsetMs: Int
    ) async -> MergedTimelineReportSummary? {
        do {
            let manifestPath = try writeSourceManifest(sources)
            let result = await runForResult(arguments: [
                "merged-timeline",
                "--source-manifest", manifestPath,
                "--global-offset-ms", "\(globalOffsetMs)",
                "--limit", "100"
            ])
            return decode(MergedTimelineReportSummary.self, from: result.output)
        } catch {
            diagnostics = [uiDiagnostic(code: "ui.merged_timeline_failed", message: error.localizedDescription)]
            return nil
        }
    }

    func preview(
        videoPath: String,
        sources: [CommentSourceSummary],
        outputPath: String,
        layoutTemplateId: String
    ) {
        Task {
            do {
                let manifestPath = try writeSourceManifest(sources)
                _ = await runForResult(arguments: [
                    "preview-export",
                    "--video", videoPath,
                    "--source-manifest", manifestPath,
                    "--out", outputPath,
                    "--layout-template-id", layoutTemplateId
                ])
            } catch {
                diagnostics = [uiDiagnostic(code: "ui.preview_failed", message: error.localizedDescription)]
            }
        }
    }

    func export(
        videoPath: String,
        sources: [CommentSourceSummary],
        outputPath: String,
        layoutTemplateId: String
    ) {
        Task {
            do {
                let manifestPath = try writeSourceManifest(sources)
                _ = await runForResult(arguments: [
                    "full-export",
                    "--video", videoPath,
                    "--source-manifest", manifestPath,
                    "--out", outputPath,
                    "--layout-template-id", layoutTemplateId
                ])
            } catch {
                diagnostics = [uiDiagnostic(code: "ui.export_failed", message: error.localizedDescription)]
            }
        }
    }

    private func run(arguments: [String]) {
        Task {
            _ = await runForResult(arguments: arguments)
        }
    }

    private func runForResult(arguments: [String]) async -> CLIResult {
        isRunningCommand = true
        lastCommandOutput = ""
        let result = await service.run(cliPath: cliPath, arguments: arguments)
        diagnostics = result.diagnostics
        lastCommandOutput = result.output
        isRunningCommand = false
        return result
    }

    private func decode<T: Decodable>(_ type: T.Type, from output: String) -> T? {
        guard let data = output.data(using: .utf8) else {
            diagnostics = [uiDiagnostic(code: "ui.invalid_cli_output", message: "CLI output was not UTF-8.")]
            return nil
        }
        do {
            return try JSONDecoder.afterLivie.decode(T.self, from: data)
        } catch {
            diagnostics = [uiDiagnostic(code: "ui.decode_failed", message: error.localizedDescription)] + diagnostics
            return nil
        }
    }

    private func writeSourceSpec(_ source: CommentSourceSummary) throws -> String {
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent("afterlivie-\(UUID().uuidString).source.json")
        let data = try JSONEncoder.afterLivie.encode(source)
        try data.write(to: url, options: .atomic)
        return url.path
    }

    private func writeSourceManifest(_ sources: [CommentSourceSummary]) throws -> String {
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent("afterlivie-\(UUID().uuidString).sources.json")
        let data = try JSONEncoder.afterLivie.encode(SourceManifestPayload(sources: sources))
        try data.write(to: url, options: .atomic)
        return url.path
    }

    private func uiDiagnostic(code: String, message: String) -> DiagnosticItem {
        DiagnosticItem(
            severity: "error",
            category: "project",
            code: code,
            message: message
        )
    }
}

private struct SourceManifestPayload: Encodable {
    var sources: [CommentSourceSummary]
}
