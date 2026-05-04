import Foundation
import SwiftUI
import UniformTypeIdentifiers

extension UTType {
    static let replayProject = UTType(exportedAs: "com.afterlivie.replayproj")
}

struct ReplayDocument: FileDocument {
    static var readableContentTypes: [UTType] { [.replayProject, .json] }
    static var writableContentTypes: [UTType] { [.replayProject] }

    var project: ReplayProjectSummary

    init(project: ReplayProjectSummary = ReplayProjectSummary()) {
        self.project = project
    }

    init(configuration: ReadConfiguration) throws {
        if let wrappers = configuration.file.fileWrappers,
           let projectWrapper = wrappers["project.json"],
           let data = projectWrapper.regularFileContents {
            project = try JSONDecoder.afterLivie.decode(ReplayProjectSummary.self, from: data)
            return
        }

        if let data = configuration.file.regularFileContents {
            project = try JSONDecoder.afterLivie.decode(ReplayProjectSummary.self, from: data)
            return
        }

        project = ReplayProjectSummary()
    }

    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper {
        let data = try JSONEncoder.afterLivie.encode(project)
        let projectWrapper = FileWrapper(regularFileWithContents: data)
        projectWrapper.preferredFilename = "project.json"

        return FileWrapper(directoryWithFileWrappers: [
            "project.json": projectWrapper,
            "sources": FileWrapper(directoryWithFileWrappers: [:]),
            "assets": FileWrapper(directoryWithFileWrappers: [:]),
            "templates": FileWrapper(directoryWithFileWrappers: [:]),
            "cache": FileWrapper(directoryWithFileWrappers: [:])
        ])
    }
}

struct ReplayProjectSummary: Codable, Equatable {
    var schemaVersion: String = "1.0"
    var projectId: String = UUID().uuidString
    var createdAt: Date = Date()
    var updatedAt: Date = Date()
    var sourceVideoPath: String?
    var commentSourcePath: String?
    var layoutTemplateId: String = "classic-sidebar-v1"
    var globalOffsetMs: Int = 0
    var diagnostics: [DiagnosticItem] = []
    var renderHistory: [RenderHistoryItem] = []
}

struct DiagnosticItem: Identifiable, Codable, Equatable {
    var id: String
    var severity: String
    var category: String
    var code: String
    var message: String
    var actionableHint: String?

    init(
        id: String = UUID().uuidString,
        severity: String,
        category: String,
        code: String,
        message: String,
        actionableHint: String? = nil
    ) {
        self.id = id
        self.severity = severity
        self.category = category
        self.code = code
        self.message = message
        self.actionableHint = actionableHint
    }
}

struct RenderHistoryItem: Identifiable, Codable, Equatable {
    var id: String = UUID().uuidString
    var outputPath: String
    var preset: String
    var completedAt: Date = Date()
}

extension JSONEncoder {
    static var afterLivie: JSONEncoder {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        encoder.dateEncodingStrategy = .iso8601
        return encoder
    }
}

extension JSONDecoder {
    static var afterLivie: JSONDecoder {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return decoder
    }
}
