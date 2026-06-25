import SwiftUI
#if canImport(AppIntents)
import AppIntents
#endif

@main
struct HP41App: App {
    @StateObject private var calculator: CalculatorModel
    @StateObject private var fileTransfers = FileTransferCoordinator()
    @StateObject private var functionEntry = FunctionEntryCoordinator()
    @StateObject private var parameterEntry = ParameterEntryCoordinator()
    @StateObject private var preferences = AppPreferences()
    @Environment(\.scenePhase) private var scenePhase

    init() {
        let calculator = CalculatorModel()
        _calculator = StateObject(wrappedValue: calculator)
        #if canImport(AppIntents)
        if #available(macOS 13.0, *) {
            AppDependencyManager.shared.add(dependency: calculator)
        }
        #endif
    }

    var body: some Scene {
        WindowGroup("HP-41 Calculator") {
            CalculatorView(
                model: calculator,
                fileTransfers: fileTransfers,
                functionEntry: functionEntry,
                parameterEntry: parameterEntry,
                preferences: preferences
            )
                .task { calculator.consumePendingAppIntent() }
                .onReceive(NotificationCenter.default.publisher(for: .hp41AppIntentEnqueued)) { _ in
                    calculator.consumePendingAppIntent()
                }
                .onChange(of: scenePhase) { _, phase in
                    calculator.setSceneActive(phase == .active)
                    if phase != .active { calculator.saveState() }
                    else { calculator.consumePendingAppIntent() }
                }
        }
        .windowResizability(.contentSize)
        .commands {
            CommandMenu("Calculator") {
                Button("ENTER") { calculator.press(id: "enter") }.keyboardShortcut(.return)
                Button("Clear X") { calculator.press(id: "clx") }.keyboardShortcut(.delete)
                Divider()
                Button("Save State") { calculator.saveState() }.keyboardShortcut("s", modifiers: .command)
                Button("Soft Reset") { calculator.resetSoft() }
                Button("Execute Function…") { functionEntry.open() }
                    .keyboardShortcut("x", modifiers: [.command, .shift])
                Button("Show Printer Tape") { calculator.isPrinterPresented = true }
                    .keyboardShortcut("p", modifiers: [.command, .shift])
                Button("Play Tone…") { _ = parameterEntry.open(for: "tone") }
                Menu("Extended Parameters") {
                    Button("ARCL…") { _ = parameterEntry.open(for: "arcl_prompt") }
                    Button("ASTO…") { _ = parameterEntry.open(for: "asto_prompt") }
                    Button("DSE…") { _ = parameterEntry.open(for: "dse_prompt") }
                    Button("DEL…") { _ = parameterEntry.open(for: "del_prompt") }
                    Button("SIZE…") { _ = parameterEntry.open(for: "size_prompt") }
                    Button("STO Arithmetic…") { _ = parameterEntry.open(for: "sto_arith_prompt") }
                    Divider()
                    Button("FC?…") { _ = parameterEntry.open(for: "fc_prompt") }
                    Button("FS?C…") { _ = parameterEntry.open(for: "fs_c_prompt") }
                    Button("FC?C…") { _ = parameterEntry.open(for: "fc_c_prompt") }
                    Button("GTO IND…") { _ = parameterEntry.open(for: "gto_ind_prompt") }
                    Button("XEQ IND…") { _ = parameterEntry.open(for: "xeq_ind_prompt") }
                }
                Divider()
                Button("Import Program…") { fileTransfers.chooseRawImport(for: calculator) }
                    .keyboardShortcut("o", modifiers: [.command, .shift])
                Button("Export Program…") { fileTransfers.chooseRawExport(for: calculator) }
                    .keyboardShortcut("e", modifiers: [.command, .shift])
                Button("Import Data Card…") { fileTransfers.chooseDataImport(for: calculator) }
                Button("Export Data Card…") { fileTransfers.chooseDataExport(for: calculator) }
            }
        }
    }
}
