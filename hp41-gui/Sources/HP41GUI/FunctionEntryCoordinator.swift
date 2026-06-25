import Foundation

@MainActor
final class FunctionEntryCoordinator: ObservableObject {
    @Published var isPresented = false
    @Published var query = ""
    @Published var alphaLabel = ""

    var filteredFunctions: [FunctionCatalogEntry] {
        FunctionCatalog.all.filter { $0.matches(query) }
    }

    func open() {
        query = ""
        alphaLabel = ""
        isPresented = true
    }

    func execute(_ name: String, on model: CalculatorModel) {
        let normalized = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalized.isEmpty else { return }
        model.press(id: "xeq_\(normalized)")
        guard model.state.error == nil else { return }
        if model.state.modalRequiresAlphaLabel {
            alphaLabel = ""
        } else {
            isPresented = false
        }
    }

    func submitAlphaLabel(on model: CalculatorModel) {
        let normalized = alphaLabel.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalized.isEmpty else { return }
        model.submitModal(label: normalized)
        alphaLabel = ""
        if !model.state.modalRequiresAlphaLabel {
            isPresented = false
        }
    }

    func cancel(on model: CalculatorModel) {
        if model.state.modalProgramActive { model.cancelModal() }
        isPresented = false
        query = ""
        alphaLabel = ""
    }
}
