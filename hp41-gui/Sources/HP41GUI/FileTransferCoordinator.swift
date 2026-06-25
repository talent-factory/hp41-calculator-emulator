import AppKit
import Foundation
import UniformTypeIdentifiers

struct RawImportSelection: Identifiable, Equatable {
    let id = UUID()
    let url: URL
    let programs: [RawProgramInfo]
}

struct FileTransferNotice: Identifiable, Equatable {
    let id = UUID()
    let title: String
    let message: String
}

@MainActor
final class FileTransferCoordinator: ObservableObject {
    @Published var pendingRawImport: RawImportSelection?
    @Published var selectedProgramIndices: Set<Int> = []
    @Published var notice: FileTransferNotice?

    private static let rawType = UTType(filenameExtension: "raw") ?? .data

    func chooseRawImport(for model: CalculatorModel) {
        let panel = NSOpenPanel()
        panel.title = "Import HP-41 Program"
        panel.allowedContentTypes = [Self.rawType]
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        guard panel.runModal() == .OK, let url = panel.url else { return }
        prepareRawImport(from: url, for: model)
    }

    func prepareRawImport(from url: URL, for model: CalculatorModel) {
        guard let programs = withSecurityScopedAccess(to: url, { model.inspectRaw(at: url) }) else {
            showModelError(model, fallback: "The RAW archive could not be inspected.")
            return
        }
        switch programs.count {
        case 0:
            notice = FileTransferNotice(
                title: "No Programs Found",
                message: "The selected RAW file contains no programs."
            )
        case 1:
            importRaw(from: url, indices: [programs[0].index], for: model)
        default:
            selectedProgramIndices = Set(programs.map(\.index))
            pendingRawImport = RawImportSelection(url: url, programs: programs)
        }
    }

    func confirmRawImport(for model: CalculatorModel) {
        guard let pendingRawImport else { return }
        let indices = pendingRawImport.programs
            .map(\.index)
            .filter(selectedProgramIndices.contains)
        guard !indices.isEmpty else {
            notice = FileTransferNotice(
                title: "No Programs Selected",
                message: "Select at least one program to import."
            )
            return
        }
        importRaw(from: pendingRawImport.url, indices: indices, for: model)
    }

    func cancelRawImport() {
        pendingRawImport = nil
        selectedProgramIndices = []
    }

    func chooseRawExport(for model: CalculatorModel) {
        let panel = NSSavePanel()
        panel.title = "Export HP-41 Program"
        panel.nameFieldStringValue = "program.raw"
        panel.allowedContentTypes = [Self.rawType]
        guard panel.runModal() == .OK, let url = panel.url else { return }
        exportRaw(to: url, for: model)
    }

    func exportRaw(to url: URL, for model: CalculatorModel) {
        if withSecurityScopedAccess(to: url, { model.exportRaw(to: url) }) != nil {
            notice = FileTransferNotice(title: "Program Exported", message: "Saved \(url.lastPathComponent).")
        } else {
            showModelError(model, fallback: "The RAW program could not be written.")
        }
    }

    func chooseDataImport(for model: CalculatorModel) {
        let panel = NSOpenPanel()
        panel.title = "Import HP-41 Data Card"
        panel.allowedContentTypes = [.json]
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        guard panel.runModal() == .OK, let url = panel.url else { return }
        importData(from: url, for: model)
    }

    func importData(from url: URL, for model: CalculatorModel) {
        if withSecurityScopedAccess(to: url, { model.importData(at: url) }) != nil {
            notice = FileTransferNotice(title: "Data Card Imported", message: "Loaded \(url.lastPathComponent).")
        } else {
            showModelError(model, fallback: "The data card could not be loaded.")
        }
    }

    func chooseDataExport(for model: CalculatorModel) {
        let panel = NSSavePanel()
        panel.title = "Export HP-41 Data Card"
        panel.nameFieldStringValue = "data.card.json"
        panel.allowedContentTypes = [.json]
        guard panel.runModal() == .OK, let url = panel.url else { return }
        exportData(to: url, for: model)
    }

    func exportData(to url: URL, for model: CalculatorModel) {
        if withSecurityScopedAccess(to: url, { model.exportData(to: url) }) != nil {
            notice = FileTransferNotice(title: "Data Card Exported", message: "Saved \(url.lastPathComponent).")
        } else {
            showModelError(model, fallback: "The data card could not be written.")
        }
    }

    private func importRaw(from url: URL, indices: [Int], for model: CalculatorModel) {
        if withSecurityScopedAccess(to: url, { model.importRaw(at: url, indices: indices) }) != nil {
            pendingRawImport = nil
            selectedProgramIndices = []
            let count = indices.count
            notice = FileTransferNotice(
                title: "Program Import Complete",
                message: "Imported \(count) program\(count == 1 ? "" : "s") from \(url.lastPathComponent)."
            )
        } else {
            showModelError(model, fallback: "The selected program could not be imported.")
        }
    }

    private func showModelError(_ model: CalculatorModel, fallback: String) {
        notice = FileTransferNotice(title: "File Transfer Failed", message: model.state.error ?? fallback)
    }

    private func withSecurityScopedAccess<Result>(to url: URL, _ operation: () -> Result) -> Result {
        let didStart = url.startAccessingSecurityScopedResource()
        defer {
            if didStart { url.stopAccessingSecurityScopedResource() }
        }
        return operation()
    }
}
