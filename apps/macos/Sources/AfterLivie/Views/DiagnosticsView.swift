import SwiftUI

struct DiagnosticsView: View {
    var diagnostics: [DiagnosticItem]

    var body: some View {
        if diagnostics.isEmpty {
            ContentUnavailableView("No Diagnostics", systemImage: "checkmark.circle")
        } else {
            VStack(alignment: .leading, spacing: 12) {
                ForEach(diagnostics) { diagnostic in
                    VStack(alignment: .leading, spacing: 6) {
                        HStack {
                            Text(diagnostic.severity.uppercased())
                                .font(.caption)
                                .fontWeight(.semibold)
                            Text(diagnostic.category)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                            Spacer()
                            Text(diagnostic.code)
                                .font(.caption.monospaced())
                                .foregroundStyle(.secondary)
                        }
                        Text(diagnostic.message)
                        if let hint = diagnostic.actionableHint {
                            Text(hint)
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                    }
                    .padding(12)
                    .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 8))
                }
            }
        }
    }
}
