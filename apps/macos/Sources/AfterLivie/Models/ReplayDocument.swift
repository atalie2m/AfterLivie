import Foundation
import SwiftUI
import UniformTypeIdentifiers

extension UTType {
    static let replayProject = UTType(exportedAs: "com.afterlivie.replayproj")
    static let jsonLines = UTType(filenameExtension: "jsonl") ?? .plainText
    static let commaSeparatedValues = UTType(filenameExtension: "csv") ?? .plainText
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
    var commentSources: [CommentSourceSummary] = []
    var layoutTemplateId: String = "classic-sidebar-v1"
    var globalOffsetMs: Int = 0
    var diagnostics: [DiagnosticItem] = []
    var renderHistory: [RenderHistoryItem] = []
    var mergedTimelineRows: [MergedTimelineRowSummary] = []
    var mergedTimelineSources: [MergedTimelineSourceSummary] = []

    var primaryCommentSourcePath: String? {
        commentSources.first?.path
    }

    init() {}

    enum CodingKeys: String, CodingKey {
        case schemaVersion
        case projectId
        case createdAt
        case updatedAt
        case sourceVideoPath
        case commentSourcePath
        case commentSources
        case layoutTemplateId
        case globalOffsetMs
        case diagnostics
        case renderHistory
        case mergedTimelineRows
        case mergedTimelineSources
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        schemaVersion = try container.decodeIfPresent(String.self, forKey: .schemaVersion) ?? "1.0"
        projectId = try container.decodeIfPresent(String.self, forKey: .projectId) ?? UUID().uuidString
        createdAt = try container.decodeIfPresent(Date.self, forKey: .createdAt) ?? Date()
        updatedAt = try container.decodeIfPresent(Date.self, forKey: .updatedAt) ?? Date()
        sourceVideoPath = try container.decodeIfPresent(String.self, forKey: .sourceVideoPath)
        commentSources = try container.decodeIfPresent([CommentSourceSummary].self, forKey: .commentSources) ?? []
        if commentSources.isEmpty,
           let legacyPath = try container.decodeIfPresent(String.self, forKey: .commentSourcePath) {
            commentSources = [CommentSourceSummary(path: legacyPath)]
        }
        layoutTemplateId = try container.decodeIfPresent(String.self, forKey: .layoutTemplateId) ?? "classic-sidebar-v1"
        globalOffsetMs = try container.decodeIfPresent(Int.self, forKey: .globalOffsetMs) ?? 0
        diagnostics = try container.decodeIfPresent([DiagnosticItem].self, forKey: .diagnostics) ?? []
        renderHistory = try container.decodeIfPresent([RenderHistoryItem].self, forKey: .renderHistory) ?? []
        mergedTimelineRows = try container.decodeIfPresent([MergedTimelineRowSummary].self, forKey: .mergedTimelineRows) ?? []
        mergedTimelineSources = try container.decodeIfPresent([MergedTimelineSourceSummary].self, forKey: .mergedTimelineSources) ?? []
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(schemaVersion, forKey: .schemaVersion)
        try container.encode(projectId, forKey: .projectId)
        try container.encode(createdAt, forKey: .createdAt)
        try container.encode(updatedAt, forKey: .updatedAt)
        try container.encodeIfPresent(sourceVideoPath, forKey: .sourceVideoPath)
        try container.encode(primaryCommentSourcePath, forKey: .commentSourcePath)
        try container.encode(commentSources, forKey: .commentSources)
        try container.encode(layoutTemplateId, forKey: .layoutTemplateId)
        try container.encode(globalOffsetMs, forKey: .globalOffsetMs)
        try container.encode(diagnostics, forKey: .diagnostics)
        try container.encode(renderHistory, forKey: .renderHistory)
        try container.encode(mergedTimelineRows, forKey: .mergedTimelineRows)
        try container.encode(mergedTimelineSources, forKey: .mergedTimelineSources)
    }
}

struct CommentSourceSummary: Identifiable, Codable, Equatable {
    var id: String { sourceId }
    var path: String
    var format: String
    var sourceId: String
    var displayName: String
    var platform: String?
    var enabled: Bool = true
    var offsetMs: Int = 0
    var csvMapping: CsvColumnMappingSummary?
    var headers: [String] = []
    var previewRows: [ImportPreviewRowSummary] = []
    var diagnostics: [DiagnosticItem] = []
    var importerId: String?
    var importerVersion: String?
    var timestampBasis: String = "relative_to_video_start"
    var commentCount: Int = 0
    var skippedCount: Int = 0
    var metadata: [String: String] = [:]

    init(path: String) {
        self.path = path
        format = Self.format(for: path)
        sourceId = Self.slug(for: path)
        displayName = URL(fileURLWithPath: path).deletingPathExtension().lastPathComponent
        platform = nil
        if format == "csv" {
            csvMapping = CsvColumnMappingSummary()
        }
    }

    mutating func apply(preview: ImportPreviewSummary) {
        importerId = preview.importerId
        importerVersion = preview.importerVersion
        headers = preview.headers
        if let mapping = preview.csvMapping {
            csvMapping = mapping
        }
        previewRows = preview.rows
        diagnostics = preview.diagnostics
        skippedCount = Int(preview.skippedCount)
        commentCount = preview.rows.count
    }

    static func format(for path: String) -> String {
        switch URL(fileURLWithPath: path).pathExtension.lowercased() {
        case "csv": "csv"
        case "jsonl", "ndjson": "jsonl"
        default: "canonical_json"
        }
    }

    static func slug(for path: String) -> String {
        let base = URL(fileURLWithPath: path)
            .deletingPathExtension()
            .lastPathComponent
            .lowercased()
        let mapped = base.unicodeScalars
            .map { scalar in CharacterSet.alphanumerics.contains(scalar) ? String(scalar) : "_" }
            .joined()
        let collapsed = mapped
            .split(separator: "_")
            .joined(separator: "_")
        return collapsed.isEmpty ? "source" : collapsed
    }
}

struct CsvColumnMappingSummary: Codable, Equatable {
    var timestamp: String = ""
    var text: String = ""
    var id: String?
    var author: String?
    var kind: String?
    var platform: String?
    var metadata: [String: String] = [:]
}

struct ImportPreviewSummary: Codable, Equatable {
    var sourceId: String
    var format: String
    var importerId: String
    var importerVersion: String
    var detectedEncoding: String?
    var headers: [String]
    var csvMapping: CsvColumnMappingSummary?
    var rows: [ImportPreviewRowSummary]
    var diagnostics: [DiagnosticItem]
    var skippedCount: UInt64
}

struct ImportPreviewRowSummary: Identifiable, Codable, Equatable {
    var id: String { "\(sourceId):\(commentId):\(rowNumber)" }
    var rowNumber: UInt64
    var sourceId: String
    var commentId: String
    var timestampMs: Int
    var authorDisplayName: String?
    var text: String
    var kind: String
    var platform: String?
}

struct MergedTimelineReportSummary: Codable, Equatable {
    var sourceSummaries: [MergedTimelineSourceSummary]
    var rows: [MergedTimelineRowSummary]
    var diagnostics: [DiagnosticItem]
}

struct MergedTimelineSourceSummary: Identifiable, Codable, Equatable {
    var id: String { sourceId }
    var sourceId: String
    var displayName: String
    var enabled: Bool
    var offsetMs: Int
    var commentCount: Int
    var skippedCount: UInt64
}

struct MergedTimelineRowSummary: Identifiable, Codable, Equatable {
    var id: String { "\(sourceId):\(commentId):\(effectiveTimestampMs)" }
    var sourceId: String
    var commentId: String
    var originalTimestampMs: Int
    var effectiveTimestampMs: Int
    var authorDisplayName: String?
    var text: String
    var kind: String
    var platform: String?
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
