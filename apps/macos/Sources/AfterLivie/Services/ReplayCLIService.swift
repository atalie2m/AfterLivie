import Foundation

struct CLIResult: Sendable {
    var exitCode: Int32
    var output: String
    var diagnostics: [DiagnosticItem]
}

actor ReplayCLIService {
    func run(cliPath: String, arguments: [String]) -> CLIResult {
        let process = Process()
        let resolved = resolveExecutable(cliPath, arguments: arguments)
        process.executableURL = resolved.executable
        process.arguments = resolved.arguments

        let stdout = Pipe()
        let stderr = Pipe()
        process.standardOutput = stdout
        process.standardError = stderr

        do {
            try process.run()
            process.waitUntilExit()
        } catch {
            return CLIResult(
                exitCode: 127,
                output: error.localizedDescription,
                diagnostics: [
                    DiagnosticItem(
                        severity: "fatal",
                        category: "project",
                        code: "ui.cli_launch_failed",
                        message: "Could not launch AfterLivie CLI.",
                        actionableHint: "Build the Rust workspace and set AFTERLIVIE_CLI or add afterlivie to PATH."
                    )
                ]
            )
        }

        let output = String(data: stdout.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
        let errorOutput = String(data: stderr.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
        let combined = [output, errorOutput].filter { !$0.isEmpty }.joined(separator: "\n")

        return CLIResult(
            exitCode: process.terminationStatus,
            output: combined,
            diagnostics: parseDiagnostics(from: combined, exitCode: process.terminationStatus)
        )
    }

    private func resolveExecutable(
        _ cliPath: String,
        arguments: [String]
    ) -> (executable: URL, arguments: [String]) {
        if cliPath.contains("/") {
            return (URL(fileURLWithPath: cliPath), arguments)
        }
        return (URL(fileURLWithPath: "/usr/bin/env"), [cliPath] + arguments)
    }

    private func parseDiagnostics(from output: String, exitCode: Int32) -> [DiagnosticItem] {
        guard let data = output.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            if exitCode == 0 {
                return []
            }
            return [
                DiagnosticItem(
                    severity: "fatal",
                    category: "project",
                    code: "ui.cli_failed",
                    message: output.isEmpty ? "CLI command failed." : output
                )
            ]
        }

        if let diagnostics = json["diagnostics"] as? [[String: Any]] {
            return diagnostics.map { item in
                DiagnosticItem(
                    id: item["id"] as? String ?? UUID().uuidString,
                    severity: item["severity"] as? String ?? "info",
                    category: item["category"] as? String ?? "project",
                    code: item["code"] as? String ?? "unknown",
                    message: item["message"] as? String ?? "",
                    actionableHint: item["actionableHint"] as? String
                )
            }
        }
        return []
    }
}
