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
    var alignmentSuggestions: [AlignmentSuggestionSummary] = []
    var orderingRules: [String] = []

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
        case alignmentSuggestions
        case orderingRules
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
        alignmentSuggestions = try container.decodeIfPresent([AlignmentSuggestionSummary].self, forKey: .alignmentSuggestions) ?? []
        orderingRules = try container.decodeIfPresent([String].self, forKey: .orderingRules) ?? []
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
        try container.encode(alignmentSuggestions, forKey: .alignmentSuggestions)
        try container.encode(orderingRules, forKey: .orderingRules)
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
    var alignmentSuggestions: [AlignmentSuggestionSummary] = []
    var orderingRules: [String] = []

    enum CodingKeys: String, CodingKey {
        case sourceSummaries
        case rows
        case diagnostics
        case alignmentSuggestions
        case orderingRules
    }

    init(
        sourceSummaries: [MergedTimelineSourceSummary],
        rows: [MergedTimelineRowSummary],
        diagnostics: [DiagnosticItem],
        alignmentSuggestions: [AlignmentSuggestionSummary] = [],
        orderingRules: [String] = []
    ) {
        self.sourceSummaries = sourceSummaries
        self.rows = rows
        self.diagnostics = diagnostics
        self.alignmentSuggestions = alignmentSuggestions
        self.orderingRules = orderingRules
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        sourceSummaries = try container.decodeIfPresent([MergedTimelineSourceSummary].self, forKey: .sourceSummaries) ?? []
        rows = try container.decodeIfPresent([MergedTimelineRowSummary].self, forKey: .rows) ?? []
        diagnostics = try container.decodeIfPresent([DiagnosticItem].self, forKey: .diagnostics) ?? []
        alignmentSuggestions = try container.decodeIfPresent([AlignmentSuggestionSummary].self, forKey: .alignmentSuggestions) ?? []
        orderingRules = try container.decodeIfPresent([String].self, forKey: .orderingRules) ?? []
    }
}

struct MergedTimelineSourceSummary: Identifiable, Codable, Equatable {
    var id: String { sourceId }
    var sourceId: String
    var displayName: String
    var platform: String?
    var enabled: Bool
    var offsetMs: Int
    var priority: Int
    var commentCount: Int
    var skippedCount: UInt64
    var firstEffectiveTimestampMs: Int?
    var lastEffectiveTimestampMs: Int?

    enum CodingKeys: String, CodingKey {
        case sourceId
        case displayName
        case platform
        case enabled
        case offsetMs
        case priority
        case commentCount
        case skippedCount
        case firstEffectiveTimestampMs
        case lastEffectiveTimestampMs
    }

    init(
        sourceId: String,
        displayName: String,
        platform: String? = nil,
        enabled: Bool,
        offsetMs: Int,
        priority: Int = 0,
        commentCount: Int,
        skippedCount: UInt64,
        firstEffectiveTimestampMs: Int? = nil,
        lastEffectiveTimestampMs: Int? = nil
    ) {
        self.sourceId = sourceId
        self.displayName = displayName
        self.platform = platform
        self.enabled = enabled
        self.offsetMs = offsetMs
        self.priority = priority
        self.commentCount = commentCount
        self.skippedCount = skippedCount
        self.firstEffectiveTimestampMs = firstEffectiveTimestampMs
        self.lastEffectiveTimestampMs = lastEffectiveTimestampMs
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        sourceId = try container.decode(String.self, forKey: .sourceId)
        displayName = try container.decode(String.self, forKey: .displayName)
        platform = try container.decodeIfPresent(String.self, forKey: .platform)
        enabled = try container.decodeIfPresent(Bool.self, forKey: .enabled) ?? true
        offsetMs = try container.decodeIfPresent(Int.self, forKey: .offsetMs) ?? 0
        priority = try container.decodeIfPresent(Int.self, forKey: .priority) ?? 0
        commentCount = try container.decodeIfPresent(Int.self, forKey: .commentCount) ?? 0
        skippedCount = try container.decodeIfPresent(UInt64.self, forKey: .skippedCount) ?? 0
        firstEffectiveTimestampMs = try container.decodeIfPresent(Int.self, forKey: .firstEffectiveTimestampMs)
        lastEffectiveTimestampMs = try container.decodeIfPresent(Int.self, forKey: .lastEffectiveTimestampMs)
    }
}

struct MergedTimelineRowSummary: Identifiable, Codable, Equatable {
    var id: String { "\(sourceId):\(commentId):\(effectiveTimestampMs)" }
    var sourceId: String
    var sourceDisplayName: String
    var commentId: String
    var originalTimestampMs: Int
    var effectiveTimestampMs: Int
    var authorDisplayName: String?
    var text: String
    var kind: String
    var platform: String?
    var platformLabel: String?
    var sourcePriority: Int

    enum CodingKeys: String, CodingKey {
        case sourceId
        case sourceDisplayName
        case commentId
        case originalTimestampMs
        case effectiveTimestampMs
        case authorDisplayName
        case text
        case kind
        case platform
        case platformLabel
        case sourcePriority
    }

    init(
        sourceId: String,
        sourceDisplayName: String? = nil,
        commentId: String,
        originalTimestampMs: Int,
        effectiveTimestampMs: Int,
        authorDisplayName: String?,
        text: String,
        kind: String,
        platform: String?,
        platformLabel: String? = nil,
        sourcePriority: Int = 0
    ) {
        self.sourceId = sourceId
        self.sourceDisplayName = sourceDisplayName ?? sourceId
        self.commentId = commentId
        self.originalTimestampMs = originalTimestampMs
        self.effectiveTimestampMs = effectiveTimestampMs
        self.authorDisplayName = authorDisplayName
        self.text = text
        self.kind = kind
        self.platform = platform
        self.platformLabel = platformLabel
        self.sourcePriority = sourcePriority
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        sourceId = try container.decode(String.self, forKey: .sourceId)
        sourceDisplayName = try container.decodeIfPresent(String.self, forKey: .sourceDisplayName) ?? sourceId
        commentId = try container.decode(String.self, forKey: .commentId)
        originalTimestampMs = try container.decode(Int.self, forKey: .originalTimestampMs)
        effectiveTimestampMs = try container.decode(Int.self, forKey: .effectiveTimestampMs)
        authorDisplayName = try container.decodeIfPresent(String.self, forKey: .authorDisplayName)
        text = try container.decode(String.self, forKey: .text)
        kind = try container.decode(String.self, forKey: .kind)
        platform = try container.decodeIfPresent(String.self, forKey: .platform)
        platformLabel = try container.decodeIfPresent(String.self, forKey: .platformLabel)
        sourcePriority = try container.decodeIfPresent(Int.self, forKey: .sourcePriority) ?? 0
    }
}

struct AlignmentSuggestionSummary: Identifiable, Codable, Equatable {
    var id: String { "\(sourceId):\(suggestedOffsetMs):\(basis)" }
    var sourceId: String
    var suggestedOffsetMs: Int
    var confidence: Double
    var basis: String
    var reason: String
    var requiresReview: Bool
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
