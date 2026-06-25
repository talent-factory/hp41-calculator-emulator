import AppKit
import SwiftUI

private struct PressHandlingButton: NSViewRepresentable {
    let tap: () -> Void
    let longPress: () -> Void

    func makeNSView(context: Context) -> PressTrackingNSButton {
        PressTrackingNSButton(tap: tap, longPress: longPress)
    }

    func updateNSView(_ view: PressTrackingNSButton, context: Context) {
        view.tap = tap
        view.longPress = longPress
    }

    final class PressTrackingNSButton: NSButton {
        var tap: () -> Void
        var longPress: () -> Void

        init(tap: @escaping () -> Void, longPress: @escaping () -> Void) {
            self.tap = tap
            self.longPress = longPress
            super.init(frame: .zero)
            title = ""
            isBordered = false
            setAccessibilityLabel("ON")
            setAccessibilityHelp("Tap for soft reset. Hold for full reset confirmation.")
            setAccessibilityIdentifier("key-on")
        }

        @available(*, unavailable)
        required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }

        override func mouseDown(with event: NSEvent) {
            guard let window else { return }
            guard let mouseUp = window.nextEvent(
                matching: .leftMouseUp,
                until: .distantFuture,
                inMode: .eventTracking,
                dequeue: true
            ) else { return }
            if mouseUp.timestamp - event.timestamp >= 0.6 { longPress() }
            else { tap() }
        }
    }
}

private enum CalculatorAlert: Identifiable {
    case fullReset
    case fileTransfer(FileTransferNotice)

    var id: String {
        switch self {
        case .fullReset: "full-reset"
        case .fileTransfer(let notice): notice.id.uuidString
        }
    }
}

struct CalculatorView: View {
    @ObservedObject var model: CalculatorModel
    @ObservedObject var fileTransfers: FileTransferCoordinator
    @ObservedObject var functionEntry: FunctionEntryCoordinator
    @ObservedObject var parameterEntry: ParameterEntryCoordinator
    @ObservedObject var preferences: AppPreferences
    @FocusState private var calculatorFocused: Bool
    @State private var activeAlert: CalculatorAlert?
    @State private var isProgramListingExpanded = false
    @State private var isSettingsPresented = false
    @State private var isOnboardingPresented = false
    @State private var onboardingIsFirstRun = false
    @State private var isHelpPresented = false

    var body: some View {
        VStack(spacing: 14) {
            brand
            display
            programPanel
            modeRow
            keyboard
            if let error = model.state.error {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .accessibilityLabel("Calculator error")
                    .accessibilityValue(error)
                    .accessibilityIdentifier("calculator-error")
            }
        }
        .padding(18)
        .frame(minWidth: 420, idealWidth: 460, minHeight: 720)
        .background(
            LinearGradient(colors: [palette.bodyTop.color, palette.bodyBottom.color], startPoint: .top, endPoint: .bottom)
        )
        .preferredColorScheme(preferences.theme == .light ? .light : .dark)
        .focusable()
        .focused($calculatorFocused)
        .onAppear {
            calculatorFocused = true
            if !preferences.onboardingDone {
                onboardingIsFirstRun = true
                isOnboardingPresented = true
            }
        }
        .onChange(of: model.state.modalRequiresAlphaLabel) { _, requiresLabel in
            if requiresLabel {
                functionEntry.alphaLabel = ""
                functionEntry.isPresented = true
            }
        }
        .onKeyPress { handlePhysicalKey($0) }
        .onKeyPress(.escape) {
            if model.isExecutionBusy {
                model.requestCancel()
                return .handled
            }
            if model.capturePendingKey(keyCode: 0) { return .handled }
            if model.state.modalProgramActive {
                model.cancelModal()
                return .handled
            }
            return .ignored
        }
        .sheet(isPresented: $functionEntry.isPresented) {
            FunctionEntrySheet(model: model, coordinator: functionEntry)
        }
        .sheet(isPresented: Binding(
            get: { parameterEntry.isPresented },
            set: { parameterEntry.isPresented = $0 }
        )) {
            ParameterEntrySheet(model: model, coordinator: parameterEntry)
        }
        .sheet(isPresented: $model.isPrinterPresented) {
            PrinterTapeView(model: model)
        }
        .sheet(isPresented: $isSettingsPresented) {
            ThemeSettingsView(
                preferences: preferences,
                isPresented: $isSettingsPresented,
                showGuide: {
                    isSettingsPresented = false
                    Task { @MainActor in
                        try? await Task.sleep(for: .milliseconds(200))
                        onboardingIsFirstRun = false
                        isOnboardingPresented = true
                    }
                }
            )
        }
        .sheet(isPresented: $isOnboardingPresented) {
            OnboardingView(
                isFirstRun: onboardingIsFirstRun,
                complete: {
                    preferences.completeOnboarding()
                    isOnboardingPresented = false
                },
                close: { isOnboardingPresented = false }
            )
            .interactiveDismissDisabled(onboardingIsFirstRun)
        }
        .sheet(isPresented: $isHelpPresented) {
            NativeHelpView(model: model, isPresented: $isHelpPresented)
        }
        .sheet(item: $fileTransfers.pendingRawImport) { selection in
            RawProgramPicker(
                selection: selection,
                selectedIndices: $fileTransfers.selectedProgramIndices,
                importAction: { fileTransfers.confirmRawImport(for: model) },
                cancelAction: fileTransfers.cancelRawImport
            )
        }
        .onReceive(fileTransfers.$notice.compactMap { $0 }) { notice in
            activeAlert = .fileTransfer(notice)
        }
        .alert(item: $activeAlert) { alert in
            switch alert {
            case .fullReset:
                Alert(
                    title: Text("MEMORY LOST"),
                    message: Text("MEMORY LOST — delete all programs, registers and files?"),
                    primaryButton: .destructive(Text("Confirm"), action: performFullReset),
                    secondaryButton: .cancel(Text("Cancel"))
                )
            case .fileTransfer(let notice):
                Alert(
                    title: Text(notice.title),
                    message: Text(notice.message),
                    dismissButton: .default(Text("OK")) { fileTransfers.notice = nil }
                )
            }
        }
        .overlay(alignment: .bottom) {
            if let notice = model.outputNotice {
                Text(notice.message)
                    .font(.callout.monospaced())
                    .padding(.horizontal, 14)
                    .padding(.vertical, 9)
                    .background(.ultraThinMaterial, in: Capsule())
                    .shadow(radius: 8)
                    .padding(.bottom, 16)
                    .onTapGesture { model.dismissOutputNotice() }
                    .accessibilityLabel("Calculator notice")
                    .accessibilityValue(notice.message)
                    .accessibilityHint("Activate to dismiss.")
                    .accessibilityAddTraits(.isButton)
                    .accessibilityIdentifier("output-notice")
                    .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
        .animation(.easeOut(duration: 0.18), value: model.outputNotice)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("calculator-root")
    }

    private var brand: some View {
        HStack {
            Text("HEWLETT·PACKARD").font(.caption.bold()).tracking(1.4)
            Spacer()
            Button("PRINT") { model.isPrinterPresented = true }
                .buttonStyle(.borderless)
                .font(.caption.bold())
                .accessibilityIdentifier("printer-open")
            Button("?") { isHelpPresented = true }
                .buttonStyle(.borderless)
                .font(.caption.bold())
                .accessibilityLabel("Help and function reference")
                .accessibilityIdentifier("help-open")
            Button { isSettingsPresented = true } label: { Image(systemName: "gearshape") }
                .buttonStyle(.borderless)
                .frame(minWidth: 28, minHeight: 28)
                .contentShape(Rectangle())
                .accessibilityLabel("Settings")
                .accessibilityIdentifier("settings-open")
            Text("41CX").font(.title2.bold()).foregroundStyle(palette.accent.color)
                .accessibilityIdentifier("calculator-model")
        }
        .foregroundStyle(palette.primaryText.color.opacity(0.9))
        .accessibilitySortPriority(5)
    }

    private var display: some View {
        VStack(spacing: 8) {
            HStack(spacing: 12) {
                annunciator("USER", model.state.annunciators.user)
                annunciator("f", model.shiftActive)
                annunciator("PRGM", model.state.annunciators.prgm)
                annunciator("ALPHA", model.state.annunciators.alpha)
                annunciator(model.state.annunciators.rad ? "RAD" : "GRAD",
                            model.state.annunciators.rad || model.state.annunciators.grad)
            }
            Text(model.displayText)
                .font(.system(size: 34, weight: .medium, design: .monospaced))
                .foregroundStyle(palette.displayText.color)
                .lineLimit(1).minimumScaleFactor(0.55)
                .frame(maxWidth: .infinity, alignment: .trailing)
                .accessibilityLabel("Display \(model.displayText)")
                .accessibilityIdentifier("calculator-display")
                .accessibilityValue(model.displayText)
            if let pendingYield = model.state.pendingYield {
                Text(pendingYield.kind == "wait_for_key" ? "Waiting for key" : "Program paused")
                    .font(.caption.bold())
                    .foregroundStyle(palette.displayText.color.opacity(0.75))
                    .accessibilityLabel(
                        pendingYield.kind == "wait_for_key"
                            ? "Program waiting for key input"
                            : "Program paused and will resume automatically"
                    )
                    .accessibilityValue(pendingYield.kind)
                    .accessibilityIdentifier("program-yield-status")
            }
            HStack(spacing: 14) {
                stack("T", model.state.t); stack("Z", model.state.z)
                stack("Y", model.state.y); stack("X", model.state.x)
            }
        }
        .padding(14)
        .background(palette.displayBackground.color, in: RoundedRectangle(cornerRadius: 9))
        .overlay(RoundedRectangle(cornerRadius: 9).stroke(.black, lineWidth: 5))
        .accessibilitySortPriority(4)
    }

    private func annunciator(_ text: String, _ active: Bool) -> some View {
        Text(text).font(.system(size: 10, weight: .bold, design: .monospaced))
            .foregroundStyle(active ? palette.displayText.color : palette.displayText.color.opacity(0.15))
            .accessibilityIdentifier("annunciator-\(text.lowercased())")
            .accessibilityValue(active ? "active" : "inactive")
    }

    private func stack(_ name: String, _ value: String) -> some View {
        Text("\(name) \(value)").font(.system(size: 9, design: .monospaced))
            .foregroundStyle(palette.displayText.color.opacity(0.58)).lineLimit(1)
            .accessibilityLabel("\(name) register")
            .accessibilityValue(value)
            .accessibilityIdentifier("stack-\(name.lowercased())")
    }

    private var programPanel: some View {
        VStack(spacing: 8) {
            Button {
                isProgramListingExpanded.toggle()
            } label: {
                HStack {
                    Image(systemName: isProgramListingExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption.bold())
                    Text("PROGRAM").font(.caption.bold()).tracking(0.8)
                    Spacer()
                    Text("PC \(formattedProgramCounter)")
                        .font(.caption.monospacedDigit())
                        .accessibilityIdentifier("program-counter")
                }
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Program listing, PC \(formattedProgramCounter)")
            .accessibilityIdentifier("program-listing-toggle")
            .accessibilityValue(isProgramListingExpanded ? "expanded" : "collapsed")

            if isProgramListingExpanded {
                Divider().overlay(.white.opacity(0.2))
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 2) {
                            ForEach(Array(model.state.programSteps.enumerated()), id: \.offset) { index, step in
                                Text(step)
                                    .font(.system(.caption, design: .monospaced))
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .padding(.horizontal, 6)
                                    .padding(.vertical, 3)
                                    .background(index == model.state.pc ? Color.orange.opacity(0.28) : .clear,
                                                in: RoundedRectangle(cornerRadius: 3))
                                    .accessibilityLabel(index == model.state.pc ? "Current step \(step)" : step)
                                    .accessibilityIdentifier("program-step-\(index)")
                                    .id(index)
                            }
                        }
                    }
                    .frame(minHeight: 44, maxHeight: 130)
                    .onChange(of: model.state.pc) { _, pc in
                        withAnimation { proxy.scrollTo(pc, anchor: .center) }
                    }
                }
            }

            HStack(spacing: 12) {
                programControl("SST", identifier: "program-sst", action: model.singleStep)
                    .disabled(model.isExecutionBusy)
                programControl("BST", identifier: "program-bst", action: model.backStep)
                    .disabled(model.isExecutionBusy)
                programControl(model.isExecutionBusy ? "STOP" : "R/S",
                               identifier: "program-run-stop", action: model.startOrStopProgramInBackground)
            }
        }
        .padding(10)
        .background(Color.white.opacity(0.055), in: RoundedRectangle(cornerRadius: 8))
        .overlay(RoundedRectangle(cornerRadius: 8).stroke(palette.trim.color.opacity(0.5)))
        .foregroundStyle(palette.primaryText.color.opacity(0.9))
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("program-panel")
        .accessibilitySortPriority(3)
    }

    private var formattedProgramCounter: String {
        let digits = String(model.state.pc)
        return String(repeating: "0", count: max(0, 3 - digits.count)) + digits
    }

    private func programControl(_ label: String, identifier: String,
                                action: @escaping () -> Void) -> some View {
        Button(label, action: action)
            .buttonStyle(.bordered)
            .controlSize(.small)
            .frame(maxWidth: .infinity)
            .accessibilityIdentifier(identifier)
    }

    private var modeRow: some View {
        HStack(spacing: 10) {
            ForEach(KeyboardLayout.modes) { key in
                if key.label == "ON" { onKeyButton(key) }
                else { keyButton(key) }
            }
        }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("calculator-mode-row")
        .accessibilitySortPriority(2)
    }

    /// ON intentionally bypasses normal calculator dispatch so it remains an
    /// escape hatch even when input, a modal, or program execution is stuck.
    private func onKeyButton(_ key: CalculatorKey) -> some View {
        ZStack {
            keyLabel(key)
            PressHandlingButton(
                tap: performSoftReset,
                longPress: { activeAlert = .fullReset }
            )
        }
        .contextMenu {
            Button("Full Reset…") { activeAlert = .fullReset }
        }
    }

    private var keyboard: some View {
        VStack(spacing: 9) {
            ForEach(Array(KeyboardLayout.rows.enumerated()), id: \.offset) { _, row in
                keyboardRow(row)
            }
        }
        .padding(12)
        .overlay(RoundedRectangle(cornerRadius: 9).stroke(palette.trim.color))
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("calculator-keyboard")
        .accessibilitySortPriority(1)
    }

    private func keyboardRow(_ row: [CalculatorKey]) -> some View {
        GeometryReader { geometry in
            let spacing: CGFloat = 9
            let totalSpacing = spacing * CGFloat(max(0, row.count - 1))
            let totalColumns = CGFloat(row.reduce(0) { $0 + $1.columnSpan })
            let columnWidth = (geometry.size.width - totalSpacing) / totalColumns
            HStack(spacing: spacing) {
                ForEach(row) { key in
                    keyButton(key)
                        .frame(width: columnWidth * CGFloat(key.columnSpan))
                }
            }
        }
        .frame(height: 52)
    }

    private func keyButton(_ key: CalculatorKey) -> some View {
        Button { activate(key) } label: {
            keyLabel(key)
        }
        .buttonStyle(.plain)
        .disabled(key.commandID.isEmpty || (model.isExecutionBusy && key.commandID != "r_s"))
        .accessibilityLabel(primaryLabel(for: key))
        .accessibilityHint(accessibilityHint(for: key))
        .accessibilityIdentifier(key.accessibilityIdentifier)
        .accessibilityValue(accessibilityValue(for: key))
    }

    private func keyLabel(_ key: CalculatorKey) -> some View {
        VStack(spacing: 2) {
            Text(shiftedLabel(for: key) ?? " ").font(.system(size: 9, weight: .bold)).foregroundStyle(.orange)
            Text(primaryLabel(for: key)).font(.system(size: key.kind == .enter ? 12 : 14, weight: .bold))
                .foregroundStyle(palette.primaryText.color)
            Text(key.alpha ?? " ").font(.system(size: 8, weight: .bold)).foregroundStyle(.blue.opacity(0.85))
        }
        .frame(maxWidth: .infinity, minHeight: 48)
        .background(keyColor(key), in: RoundedRectangle(cornerRadius: 6))
        .contentShape(Rectangle())
    }

    private func primaryLabel(for key: CalculatorKey) -> String {
        KeyboardLayout.primaryLabel(
            for: key,
            userActive: model.state.annunciators.user,
            assignments: model.state.userKeymap
        )
    }

    private func accessibilityHint(for key: CalculatorKey) -> String {
        var hints: [String] = []
        if primaryLabel(for: key) != key.label { hints.append("Assigned key; original function \(key.label)") }
        if let shifted = key.shiftedLabel { hints.append("Shifted: \(shifted)") }
        return hints.joined(separator: ". ")
    }

    private func accessibilityValue(for key: CalculatorKey) -> String {
        switch key.commandID {
        case "shift": model.shiftActive ? "active" : "inactive"
        case "user_mode": model.state.annunciators.user ? "active" : "inactive"
        case "prgm_mode": model.state.annunciators.prgm ? "active" : "inactive"
        case "alpha_toggle": model.state.annunciators.alpha ? "active" : "inactive"
        case "r_s": model.isExecutionBusy || model.state.isRunning ? "running" : "stopped"
        default: ""
        }
    }

    private func dismissInputForReset() {
        functionEntry.isPresented = false
        parameterEntry.isPresented = false
        model.shiftActive = false
    }

    private func performSoftReset() {
        dismissInputForReset()
        if model.isExecutionBusy { model.requestCancel() }
        else { model.resetSoft() }
    }

    private func performFullReset() {
        dismissInputForReset()
        if model.isExecutionBusy { model.requestCancel() }
        else { model.resetFull() }
    }

    private func activate(_ key: CalculatorKey) {
        if model.capturePendingKey(keyCode: key.keyCode) { return }
        if key.commandID == "clx_or_a" {
            model.shiftActive = false
            model.press(id: model.state.annunciators.alpha ? "alpha_backspace" : "entry_backspace")
            return
        }
        if model.state.annunciators.alpha {
            model.press(key)
            return
        }
        let shiftedID = model.state.annunciators.prgm
            ? (key.shiftedInProgramID ?? key.shiftedID)
            : key.shiftedID
        let effectiveID = model.shiftActive ? shiftedID : key.commandID
        if let effectiveID, parameterEntry.dispatchDirect(effectiveID, on: model) {
            model.shiftActive = false
            return
        }
        if let effectiveID, parameterEntry.open(for: effectiveID) {
            model.shiftActive = false
            return
        }
        if !model.shiftActive, key.commandID == "xeq_prompt" {
            functionEntry.open()
            return
        }
        if !model.shiftActive, key.commandID == "r_s", model.state.modalProgramActive {
            model.submitModal()
            return
        }
        if !model.shiftActive, key.commandID == "r_s" {
            model.startOrStopProgramInBackground()
            return
        }
        model.press(key)
    }

    private func shiftedLabel(for key: CalculatorKey) -> String? {
        if model.state.annunciators.prgm {
            return key.shiftedInProgramLabel ?? key.shiftedLabel
        }
        return key.shiftedLabel
    }

    private func keyColor(_ key: CalculatorKey) -> Color {
        switch key.kind {
        case .shift: return model.shiftActive ? palette.shiftActive.color : palette.shift.color
        case .enter: return palette.enter.color
        case .mode: return palette.key.color.opacity(0.88)
        case .normal: return palette.key.color
        }
    }

    private var palette: AppThemePalette { preferences.theme.palette }

    private func handlePhysicalKey(_ press: KeyPress) -> KeyPress.Result {
        if press.characters == "?", !model.state.annunciators.alpha {
            isHelpPresented = true
            return .handled
        }
        if press.characters == "\t" {
            model.shiftActive.toggle()
            return .handled
        }
        if model.isExecutionBusy { return .handled }
        guard let action = PhysicalKeyboardRouter.route(
            key: press.characters,
            commandOrControl: press.modifiers.contains(.command) || press.modifiers.contains(.control),
            alphaActive: model.state.annunciators.alpha,
            eexActive: model.state.inEEXMode
        ) else { return .ignored }

        switch action {
        case .saveState:
            model.saveState()
        case .dispatch(let id):
            if model.capturePendingKey(keyCode: hardwareKeyCode(for: id)) { return .handled }
            model.press(id: id)
        }
        return .handled
    }

    private func hardwareKeyCode(for commandID: String) -> Int? {
        (KeyboardLayout.modes + KeyboardLayout.rows.flatMap { $0 })
            .first(where: { $0.commandID == commandID })?.keyCode
    }
}

private struct ThemeSettingsView: View {
    @ObservedObject var preferences: AppPreferences
    @Binding var isPresented: Bool
    let showGuide: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            HStack {
                Text("Settings").font(.title2.bold())
                Spacer()
                Button("Done") { isPresented = false }
                    .keyboardShortcut(.defaultAction)
                    .accessibilityIdentifier("settings-done")
            }
            Text("Theme").font(.headline)
            Picker("Theme", selection: Binding(
                get: { preferences.theme },
                set: preferences.setTheme
            )) {
                ForEach(AppTheme.allCases) { theme in Text(theme.title).tag(theme) }
            }
            .pickerStyle(.radioGroup)
            .accessibilityIdentifier("theme-picker")
            .accessibilityValue(preferences.theme.title)
            .accessibilityHint("Changes the calculator color theme immediately.")
            Divider()
            Text("Quick Start").font(.headline)
            Button("Show Guide", action: showGuide)
                .accessibilityIdentifier("show-guide")
            Spacer()
        }
        .padding(24)
        .frame(minWidth: 360, minHeight: 280)
        .onExitCommand { isPresented = false }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("settings-sheet")
    }
}

struct OnboardingPage: Identifiable, Equatable {
    let id: Int
    let symbol: String
    let title: String
    let message: String

    static let all = [
        OnboardingPage(id: 0, symbol: "sum", title: "Welcome to HP-41CX",
                       message: "A native, programmable RPN calculator. Your calculator memory is saved automatically."),
        OnboardingPage(id: 1, symbol: "arrow.up.arrow.down", title: "RPN Basics",
                       message: "Enter the first number, press ENTER↑, enter the second number, then choose an operation."),
        OnboardingPage(id: 2, symbol: "function", title: "SHIFT and ALPHA",
                       message: "SHIFT selects the orange function once. ALPHA turns the blue key legends into text entry."),
        OnboardingPage(id: 3, symbol: "list.number", title: "Programs",
                       message: "Use PRGM to record, SST and BST to inspect steps, and R/S to run or stop a program."),
        OnboardingPage(id: 4, symbol: "externaldrive", title: "Programs and Data",
                       message: "Import or export RAW programs and data cards from the Calculator menu. Open Settings to show this guide again."),
    ]
}

private struct OnboardingView: View {
    let isFirstRun: Bool
    let complete: () -> Void
    let close: () -> Void
    @State private var step = 0

    private var page: OnboardingPage { OnboardingPage.all[step] }

    var body: some View {
        VStack(spacing: 22) {
            HStack {
                Text("Quick Start").font(.headline)
                Spacer()
                if !isFirstRun {
                    Button("Close", action: close)
                        .keyboardShortcut(.cancelAction)
                        .accessibilityIdentifier("onboarding-close")
                }
            }
            Spacer()
            Image(systemName: page.symbol)
                .font(.system(size: 46))
                .foregroundStyle(.orange)
                .accessibilityHidden(true)
            Text(page.title)
                .font(.title.bold())
                .multilineTextAlignment(.center)
                .accessibilityLabel("Quick Start page title")
                .accessibilityValue(page.title)
                .accessibilityIdentifier("onboarding-title")
            Text(page.message)
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 430)
                .accessibilityLabel("Quick Start instructions")
                .accessibilityValue(page.message)
                .accessibilityIdentifier("onboarding-message")
            Spacer()
            Text("\(step + 1) of \(OnboardingPage.all.count)")
                .font(.caption.monospacedDigit())
                .foregroundStyle(.secondary)
                .accessibilityLabel("Quick Start progress")
                .accessibilityValue("Page \(step + 1) of \(OnboardingPage.all.count)")
                .accessibilityIdentifier("onboarding-progress")
            HStack {
                Button("Back") { step -= 1 }
                    .disabled(step == 0)
                    .accessibilityIdentifier("onboarding-back")
                Spacer()
                if step == OnboardingPage.all.count - 1 {
                    Button("Get Started", action: complete)
                        .buttonStyle(.borderedProminent)
                        .accessibilityIdentifier("onboarding-finish")
                } else {
                    Button("Next") { step += 1 }
                        .buttonStyle(.borderedProminent)
                        .keyboardShortcut(.defaultAction)
                        .accessibilityIdentifier("onboarding-next")
                }
            }
        }
        .padding(30)
        .frame(minWidth: 520, minHeight: 410)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("onboarding-guide")
    }
}

private struct NativeHelpView: View {
    @ObservedObject var model: CalculatorModel
    @Binding var isPresented: Bool
    @State private var query = ""
    @State private var tab = 0
    @State private var selectedModule = "All Modules"
    @State private var expandedEntries: Set<String> = []

    private var functions: [HelpCatalogEntry] {
        HelpCatalog.all.filter { entry in
            (selectedModule == "All Modules" || entry.module == selectedModule) && entry.matches(query)
        }
    }

    private var categories: [String] {
        Array(Set(functions.map(\.category))).sorted()
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("HP-41 Reference").font(.title2.bold())
                Spacer()
                Button("Close") { isPresented = false }
                    .keyboardShortcut(.cancelAction)
                    .accessibilityIdentifier("help-close")
            }
            .padding()

            TextField("Function, category, alias, or shortcut", text: $query)
                .textFieldStyle(.roundedBorder)
                .padding(.horizontal)
                .accessibilityLabel("Search reference")
                .accessibilityIdentifier("help-search")

            Picker("Reference section", selection: $tab) {
                Text("All Functions")
                    .accessibilityIdentifier("help-tab-functions")
                    .tag(0)
                Text("Keyboard Shortcuts")
                    .accessibilityIdentifier("help-tab-shortcuts")
                    .tag(1)
            }
            .pickerStyle(.segmented)
            .padding(.horizontal)
            .accessibilityIdentifier("help-tab")
            .accessibilityValue(tab == 0 ? "All Functions" : "Keyboard Shortcuts")

            if tab == 0 {
                Picker("Module", selection: $selectedModule) {
                    Text("All Modules").tag("All Modules")
                    ForEach(HelpCatalog.modules, id: \.self) { Text($0).tag($0) }
                }
                .padding()
                .accessibilityIdentifier("help-module-filter")
                .accessibilityValue(selectedModule)
                List {
                    ForEach(categories, id: \.self) { category in
                        Section(category) {
                            ForEach(functions.filter { $0.category == category }) { entry in
                                helpEntry(entry)
                            }
                        }
                    }
                }
            } else {
                List(HelpCatalog.shortcuts.filter { $0.matches(query) }) { shortcut in
                    HStack(alignment: .top, spacing: 14) {
                        Text(shortcut.key)
                            .font(.body.monospaced().bold())
                            .frame(width: 82, alignment: .leading)
                        VStack(alignment: .leading, spacing: 3) {
                            Text(shortcut.operation).font(.headline)
                            Text(shortcut.description).foregroundStyle(.secondary)
                        }
                    }
                }
            }
        }
        .frame(minWidth: 680, minHeight: 620)
        .onExitCommand { isPresented = false }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("help-reference")
    }

    private func helpEntry(_ entry: HelpCatalogEntry) -> some View {
        let isExpanded = expandedEntries.contains(entry.opVariant)
        return VStack(alignment: .leading, spacing: 8) {
            Button {
                if isExpanded { expandedEntries.remove(entry.opVariant) }
                else { expandedEntries.insert(entry.opVariant) }
            } label: {
                HStack {
                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                    Text(entry.name).font(.body.monospaced().bold())
                    Spacer()
                    if let keyPath = entry.keyPath {
                        Text(keyPath).font(.caption.monospaced()).foregroundStyle(.secondary)
                    } else if !entry.runnable {
                        Text("parameterized").font(.caption).foregroundStyle(.secondary)
                    }
                }
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("help-entry-\(entry.opVariant)")
            .accessibilityValue(isExpanded ? "expanded" : "collapsed")

            if isExpanded {
                VStack(alignment: .leading, spacing: 8) {
                Text(entry.description)
                if let example = entry.example {
                    LabeledContent("Example", value: example).font(.callout.monospaced())
                }
                if let notes = entry.notes {
                    LabeledContent("Notes", value: notes).font(.callout)
                }
                if entry.runnable {
                    Button("Run \(entry.name)") {
                        model.press(id: "xeq_\(entry.name)")
                        if model.state.error == nil { isPresented = false }
                    }
                    .buttonStyle(.borderedProminent)
                    .accessibilityIdentifier("help-run-\(entry.opVariant)")
                }
                }
                .padding(.vertical, 6)
            }
        }
    }
}

private struct PrinterTapeView: View {
    @ObservedObject var model: CalculatorModel

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Printer Tape").font(.title2.bold())
                Spacer()
                Button("Clear") { model.clearPrintLog() }
                    .disabled(model.printLog.isEmpty)
                    .accessibilityIdentifier("printer-clear")
                Button("Close") { model.isPrinterPresented = false }
                    .keyboardShortcut(.cancelAction)
                    .accessibilityIdentifier("printer-close")
            }
            .padding()

            Divider()

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 3) {
                        if model.printLog.isEmpty {
                            Text("No printer output yet")
                                .foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, minHeight: 240)
                                .accessibilityIdentifier("printer-empty")
                        } else {
                            ForEach(Array(model.printLog.enumerated()), id: \.offset) { index, line in
                                Text(line)
                                    .font(.system(.body, design: .monospaced))
                                    .textSelection(.enabled)
                                    .accessibilityLabel("Printer line \(index + 1)")
                                    .accessibilityValue(line)
                                    .accessibilityIdentifier("printer-line-\(index)")
                                    .id(index)
                            }
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(18)
                }
                .onChange(of: model.printLog.count) { _, count in
                    if count > 0 { proxy.scrollTo(count - 1, anchor: .bottom) }
                }
            }
            .background(Color(white: 0.94))
            .foregroundStyle(.black)
        }
        .frame(minWidth: 420, minHeight: 360)
        .onExitCommand { model.isPrinterPresented = false }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("printer-tape")
    }
}

private struct ParameterEntrySheet: View {
    @ObservedObject var model: CalculatorModel
    @ObservedObject var coordinator: ParameterEntryCoordinator
    @FocusState private var inputFocused: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text(coordinator.kind?.title ?? "Parameter").font(.title2.bold())
            if isRegisterLike {
                TextField(registerPrompt, text: $coordinator.input)
                    .textFieldStyle(.roundedBorder)
                    .focused($inputFocused)
                    .onSubmit { coordinator.submit(on: model) }
                    .accessibilityIdentifier("parameter-register")
                Toggle("Indirect register", isOn: $coordinator.indirect)
                    .accessibilityIdentifier("parameter-indirect")
            } else if case .stoArithmetic = coordinator.kind {
                Picker("Operation", selection: $coordinator.arithmeticOperation) {
                    Text("+").tag("plus")
                    Text("−").tag("minus")
                    Text("×").tag("mul")
                    Text("÷").tag("div")
                }
                .accessibilityIdentifier("parameter-operation")
                TextField("Register 00–99 or Y/Z/T/LASTX", text: $coordinator.input)
                    .textFieldStyle(.roundedBorder)
                    .focused($inputFocused)
                    .onSubmit { coordinator.submit(on: model) }
                    .accessibilityIdentifier("parameter-arithmetic-register")
                Toggle("Indirect register", isOn: $coordinator.indirect)
                    .accessibilityIdentifier("parameter-arithmetic-indirect")
            } else if case .assignment = coordinator.kind {
                Picker("Target key", selection: $coordinator.assignmentKeyCode) {
                    ForEach(assignmentTargets, id: \.keyCode) { key in
                        Text("\(key.label)  [\(key.keyCode ?? 0)]").tag(key.keyCode ?? 0)
                    }
                }
                .accessibilityIdentifier("parameter-assignment-key")
                TextField("Function or program label", text: $coordinator.input)
                    .textFieldStyle(.roundedBorder)
                    .focused($inputFocused)
                    .onSubmit { coordinator.submit(on: model) }
                    .accessibilityIdentifier("parameter-assignment-label")
            } else {
                TextField("Label", text: $coordinator.input)
                    .textFieldStyle(.roundedBorder)
                    .focused($inputFocused)
                    .onSubmit { coordinator.submit(on: model) }
                    .accessibilityIdentifier("parameter-label")
            }
            if let error = model.state.error {
                Text(error).font(.caption).foregroundStyle(.red)
            }
            HStack {
                Button("Cancel") { coordinator.cancel() }
                    .keyboardShortcut(.cancelAction)
                    .accessibilityIdentifier("parameter-cancel")
                Spacer()
                Button("Enter") { coordinator.submit(on: model) }
                    .keyboardShortcut(.defaultAction)
                    .disabled(coordinator.input.isEmpty)
                    .accessibilityIdentifier("parameter-confirm")
            }
        }
        .padding(20)
        .frame(minWidth: 360)
        .onAppear { inputFocused = true }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("parameter-sheet")
    }

    private var assignmentTargets: [CalculatorKey] {
        KeyboardLayout.rows.flatMap { $0 }.filter { $0.keyCode != nil }
    }

    private var isRegisterLike: Bool {
        if case .register = coordinator.kind { return true }
        if case .flag = coordinator.kind { return true }
        return false
    }

    private var registerPrompt: String {
        if case .flag = coordinator.kind { return "Flag 00–55" }
        return "Register 00–99"
    }
}

private struct FunctionEntrySheet: View {
    @ObservedObject var model: CalculatorModel
    @ObservedObject var coordinator: FunctionEntryCoordinator
    @FocusState private var inputFocused: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Text(model.state.modalRequiresAlphaLabel ? "Function Label" : "Execute Function")
                    .font(.title2.bold())
                Spacer()
                Button("Cancel") { coordinator.cancel(on: model) }
                    .keyboardShortcut(.cancelAction)
                    .accessibilityIdentifier("function-cancel")
            }

            if model.state.modalRequiresAlphaLabel {
                Text(model.state.modalPrompt ?? "FUNCTION NAME?")
                    .foregroundStyle(.secondary)
                TextField("ALPHA label", text: $coordinator.alphaLabel)
                    .textFieldStyle(.roundedBorder)
                    .focused($inputFocused)
                    .onSubmit { coordinator.submitAlphaLabel(on: model) }
                    .accessibilityIdentifier("function-alpha-label")
                Button("Continue") { coordinator.submitAlphaLabel(on: model) }
                    .keyboardShortcut(.defaultAction)
                    .disabled(coordinator.alphaLabel.trimmingCharacters(in: .whitespaces).isEmpty)
                    .accessibilityIdentifier("function-continue")
            } else {
                TextField("Search or enter a program label", text: $coordinator.query)
                    .textFieldStyle(.roundedBorder)
                    .focused($inputFocused)
                    .onSubmit { coordinator.execute(coordinator.query, on: model) }
                    .accessibilityIdentifier("function-search")
                List(coordinator.filteredFunctions) { function in
                    Button {
                        coordinator.execute(function.name, on: model)
                    } label: {
                        VStack(alignment: .leading, spacing: 3) {
                            HStack {
                                Text(function.name).font(.headline.monospaced())
                                Spacer()
                                Text(function.category).font(.caption).foregroundStyle(.secondary)
                            }
                            Text(function.description)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(2)
                        }
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .accessibilityIdentifier("function-\(function.name)")
                }
                .frame(minHeight: 360)
            }
        }
        .padding(20)
        .frame(minWidth: 560, minHeight: 480)
        .onAppear { inputFocused = true }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("function-sheet")
    }
}

private struct RawProgramPicker: View {
    let selection: RawImportSelection
    @Binding var selectedIndices: Set<Int>
    let importAction: () -> Void
    let cancelAction: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Import Programs").font(.title2.bold())
            Text(selection.url.lastPathComponent).foregroundStyle(.secondary)
            List(selection.programs) { program in
                Toggle(isOn: binding(for: program.index)) {
                    VStack(alignment: .leading) {
                        Text(program.label)
                        Text("\(program.stepCount) steps · \(program.byteLength) bytes")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .toggleStyle(.checkbox)
                .accessibilityIdentifier("raw-program-\(program.index)")
            }
            .frame(minHeight: 220)
            HStack {
                Button("Cancel", action: cancelAction)
                    .keyboardShortcut(.cancelAction)
                    .accessibilityIdentifier("raw-import-cancel")
                Spacer()
                Button("Import Selected", action: importAction)
                    .keyboardShortcut(.defaultAction)
                    .disabled(selectedIndices.isEmpty)
                    .accessibilityIdentifier("raw-import-confirm")
            }
        }
        .padding(20)
        .frame(minWidth: 430, minHeight: 340)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("raw-import-sheet")
    }

    private func binding(for index: Int) -> Binding<Bool> {
        Binding(
            get: { selectedIndices.contains(index) },
            set: { selected in
                if selected { selectedIndices.insert(index) }
                else { selectedIndices.remove(index) }
            }
        )
    }
}
