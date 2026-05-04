import SwiftUI

struct DetailView: View {
    @Binding var document: ReplayDocument
    @EnvironmentObject private var appModel: AppModel
    var selection: SidebarSection

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            toolbar
            Divider()
            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    content
                }
                .padding(24)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    @ViewBuilder
    private var content: some View {
        switch selection {
        case .project:
            ProjectSummaryView(project: document.project)
        case .importMedia:
            ImportView(document: $document)
        case .preview:
            RenderActionView(
                title: "Preview Segment",
                actionTitle: "Render Preview",
                action: renderPreview
            )
        case .export:
            RenderActionView(
                title: "Full Export",
                actionTitle: "Export",
                action: renderExport
            )
        case .diagnostics:
            DiagnosticsView(
                project: $document.project,
                diagnostics: appModel.diagnostics + document.project.diagnostics,
                refreshTimeline: refreshMergedTimeline
            )
        }
    }

    private var toolbar: some View {
        HStack(spacing: 12) {
            Text(selection.title)
                .font(.title3)
                .fontWeight(.semibold)
            Spacer()
            if appModel.isRunningCommand {
                ProgressView()
                    .controlSize(.small)
            }
            Button("Doctor") {
                appModel.runDoctor()
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 12)
    }

    private func renderPreview() {
        guard let video = document.project.sourceVideoPath,
              !document.project.commentSources.isEmpty else {
            document.project.diagnostics.append(DiagnosticItem(
                severity: "error",
                category: "project",
                code: "ui.missing_inputs",
                message: "Select a video and at least one comment source before rendering."
            ))
            return
        }
        let output = NSTemporaryDirectory() + "afterlivie-preview.mp4"
        appModel.preview(videoPath: video, sources: document.project.commentSources, outputPath: output)
        document.project.renderHistory.append(RenderHistoryItem(outputPath: output, preset: "preview"))
    }

    private func renderExport() {
        guard let video = document.project.sourceVideoPath,
              !document.project.commentSources.isEmpty else {
            document.project.diagnostics.append(DiagnosticItem(
                severity: "error",
                category: "project",
                code: "ui.missing_inputs",
                message: "Select a video and at least one comment source before exporting."
            ))
            return
        }
        let output = NSHomeDirectory() + "/Desktop/AfterLivieExport.mp4"
        appModel.export(videoPath: video, sources: document.project.commentSources, outputPath: output)
        document.project.renderHistory.append(RenderHistoryItem(outputPath: output, preset: "high_quality_upload"))
    }

    private func refreshMergedTimeline() {
        Task {
            if let report = await appModel.mergedTimeline(
                sources: document.project.commentSources,
                globalOffsetMs: document.project.globalOffsetMs
            ) {
                document.project.mergedTimelineRows = report.rows
                document.project.mergedTimelineSources = report.sourceSummaries
                document.project.diagnostics = report.diagnostics
                for summary in report.sourceSummaries {
                    if let index = document.project.commentSources.firstIndex(where: { $0.sourceId == summary.sourceId }) {
                        document.project.commentSources[index].commentCount = summary.commentCount
                        document.project.commentSources[index].skippedCount = Int(summary.skippedCount)
                    }
                }
            }
        }
    }
}
