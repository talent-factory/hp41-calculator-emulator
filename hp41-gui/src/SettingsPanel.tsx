// Phase 48 Plan 03 — Settings panel component (D-48.1 / D-48.2 / D-48.4).
//
// Renders a popover with theme radio buttons. Click-outside closes the panel
// (no explicit close button, per D-48.4). Returns null when closed to avoid
// DOM overhead (matches HelpOverlay early-return pattern).
//
// Phase 49 Plan 04 — Extended with Quick Start section (D-48.3 / D-49.8 / D-49.9):
// - onShowOnboarding prop added to SettingsPanelProps
// - Quick Start section with "Show Guide" button added below Theme section
//
// Phase 50 — Launch Mode section (macOS-only, ADR-v4.1-001):
// - isMacos, currentLaunchMode, onLaunchModeChange props added
// - "Launch Mode (macOS)" radio section rendered only when isMacos=true
// - Selecting a mode reveals a "Restart now" affordance (invoke restart_app)

import { useRef, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { formatAccel } from './ShortcutRecorder';

export type SettingsPanelProps = {
    open: boolean;
    onClose: () => void;
    currentTheme: string;
    onThemeChange: (theme: string) => void;
    onShowOnboarding: () => void;  // D-49.8 / D-49.9: re-open wizard from settings
    isMacos: boolean;
    currentLaunchMode: string;            // "menu-bar" | "window"
    onLaunchModeChange: (mode: string) => void;
    // macOS global-hotkey config (optional so existing call sites/tests need no change).
    globalShortcut?: string;              // accelerator, e.g. "Control+Alt+Command+H"
    onRecordShortcut?: () => void;        // open the ShortcutRecorder overlay
};

const THEMES = [
    { id: 'dark', label: 'Dark' },
    { id: 'light', label: 'Light' },
    { id: 'classic-beige', label: 'Classic Beige' },
    { id: 'high-contrast', label: 'High Contrast' },
] as const;

const LAUNCH_MODES = [
    { id: 'menu-bar', label: 'Menu Bar' },
    { id: 'window', label: 'Window' },
] as const;

export function SettingsPanel({
    open, onClose, currentTheme, onThemeChange, onShowOnboarding,
    isMacos, currentLaunchMode, onLaunchModeChange,
    globalShortcut, onRecordShortcut,
}: SettingsPanelProps) {
    const panelRef = useRef<HTMLDivElement>(null);
    // Resets to false on each open: the component unmounts (returns null) when open=false.
    const [launchModeChanged, setLaunchModeChanged] = useState(false);

    // Click-outside dismiss — only register listener when open (D-48.4).
    // Use `mousedown` (not `click`) so the gear-button's `onMouseDown` with
    // stopPropagation fires before this listener, preventing immediate re-close.
    useEffect(() => {
        if (!open) return;
        const handler = (e: MouseEvent) => {
            if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
                onClose();
            }
        };
        document.addEventListener('mousedown', handler);
        return () => document.removeEventListener('mousedown', handler);
    }, [open, onClose]);

    if (!open) return null;

    return (
        <div
            ref={panelRef}
            className="settings-panel"
            role="dialog"
            aria-label="Settings"
        >
            <section className="settings-section">
                <h3 className="settings-section-heading">Theme</h3>
                {THEMES.map(t => (
                    <label key={t.id} className="settings-radio-row">
                        <input
                            type="radio"
                            name="theme"
                            value={t.id}
                            checked={currentTheme === t.id}
                            onChange={() => onThemeChange(t.id)}
                        />
                        {t.label}
                    </label>
                ))}
            </section>
            <hr className="settings-section-divider" />
            <section className="settings-section">
                <h3 className="settings-section-heading">Quick Start</h3>
                {/* D-49.9: clicking "Show Guide" closes settings first, then opens the onboarding wizard */}
                <button
                    className="settings-action-btn"
                    onClick={() => { onClose(); onShowOnboarding(); }}
                >
                    Show Guide
                </button>
            </section>
            {isMacos && (
                <>
                    <hr className="settings-section-divider" />
                    <section className="settings-section">
                        <h3 className="settings-section-heading">Launch Mode (macOS)</h3>
                        {LAUNCH_MODES.map(m => (
                            <label key={m.id} className="settings-radio-row">
                                <input
                                    type="radio"
                                    name="launch-mode"
                                    value={m.id}
                                    checked={currentLaunchMode === m.id}
                                    onChange={() => { onLaunchModeChange(m.id); setLaunchModeChanged(true); }}
                                />
                                {m.label}
                            </label>
                        ))}
                        {launchModeChanged && (
                            <div className="settings-restart-hint">
                                <span>Takes effect after restart.</span>
                                <button
                                    className="settings-action-btn"
                                    onClick={() => { invoke('restart_app').catch(() => {}); }}
                                >
                                    Restart now
                                </button>
                            </div>
                        )}
                    </section>
                    <hr className="settings-section-divider" />
                    <section className="settings-section">
                        <h3 className="settings-section-heading">Global Shortcut (macOS)</h3>
                        {/* Opens the ShortcutRecorder overlay (rendered by App.tsx). The
                            backend validates + registers the captured combo, so only an
                            accepted accelerator is reflected back into `globalShortcut`. */}
                        <button
                            className="settings-action-btn"
                            onClick={() => onRecordShortcut?.()}
                        >
                            Show/Hide window: {formatAccel(globalShortcut ?? '') || 'Set shortcut'}
                        </button>
                    </section>
                </>
            )}
            <hr className="settings-section-divider" />
            <section className="settings-section">
                <h3 className="settings-section-heading">About</h3>
                <p className="settings-about-text">
                    RPN-41 Calculator is an independent, open-source (MIT) emulation of the
                    HP-41C/CV/CX. It is not affiliated with, endorsed by or sponsored by
                    Hewlett-Packard or HP Inc. &quot;HP&quot; and &quot;HP-41&quot; are trademarks
                    of their respective owners and are used for identification only.
                </p>
            </section>
        </div>
    );
}

export default SettingsPanel;
