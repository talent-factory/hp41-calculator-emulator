// Phase 48 Plan 03 — Vitest unit tests for SettingsPanel (D-48.1 / D-48.2 / D-48.4).
//
// Tests cover:
//   1. Null render when open=false
//   2. Renders 4 theme radio buttons when open=true
//   3. Dark radio is checked when currentTheme is "dark"
//   4. Calls onThemeChange when a radio is clicked
//   5. Displays correct theme labels (Dark, Light, Classic Beige, High Contrast)
//   6. Has correct accessibility attributes (role="dialog", aria-label="Settings")
//
// Phase 49 Plan 04 — Extended tests for Quick Start section (D-49.8 / D-49.9):
//   7. Renders Quick Start section with "Show Guide" button
//   8. Clicking "Show Guide" calls onClose then onShowOnboarding
//   (All existing tests updated with required onShowOnboarding prop)
//
// Phase 50 — Launch Mode section (macOS-only, isMacos prop):
//   9.  Hides launch-mode section when isMacos=false
//  10.  Shows launch-mode radios when isMacos=true
//  11.  Calls onLaunchModeChange when Window is selected
//  12.  Shows restart hint after a launch-mode change
//
// Tauri invoke is mocked via vi.mock('@tauri-apps/api/core') following the
// HelpOverlay.test.tsx pattern.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/react';
import { SettingsPanel } from './SettingsPanel';

// Ensure DOM is cleaned up between tests so getByLabelText doesn't find
// stale elements from previous renders (auto-cleanup requires Vitest globals,
// which are disabled in this project's vitest config).
afterEach(cleanup);

// Mock Tauri API so tests run in the browser-less Vitest environment.
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn().mockResolvedValue(undefined),
}));

describe('SettingsPanel', () => {
    it('renders null when open=false', () => {
        const { container } = render(
            <SettingsPanel
                open={false}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        expect(container.firstChild).toBeNull();
    });

    it('renders 4 theme radio buttons when open=true', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        const radios = container.querySelectorAll('input[type="radio"]');
        expect(radios.length).toBe(4);
    });

    it('dark radio is checked when currentTheme is dark', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        const darkRadio = container.querySelector('input[type="radio"][value="dark"]') as HTMLInputElement;
        expect(darkRadio).not.toBeNull();
        expect(darkRadio.checked).toBe(true);
    });

    it('calls onThemeChange when a radio is clicked', () => {
        const mockFn = vi.fn();
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={mockFn}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        const lightRadio = container.querySelector('input[type="radio"][value="light"]') as HTMLInputElement;
        expect(lightRadio).not.toBeNull();
        fireEvent.click(lightRadio);
        expect(mockFn).toHaveBeenCalledWith('light');
    });

    it('displays correct theme labels', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        const text = container.textContent ?? '';
        expect(text).toContain('Dark');
        expect(text).toContain('Light');
        expect(text).toContain('Classic Beige');
        expect(text).toContain('High Contrast');
    });

    it('has correct accessibility attributes', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        const panel = container.querySelector('[role="dialog"]') as HTMLElement;
        expect(panel).not.toBeNull();
        expect(panel.getAttribute('aria-label')).toBe('Settings');
    });

    it('shows the not-affiliated-with-HP disclaimer in the About section', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        const text = container.textContent ?? '';
        expect(text).toContain('About');
        expect(text).toContain('not affiliated with');
    });

    // Phase 49 Plan 04 — Quick Start section tests (D-49.8 / D-49.9)

    it('renders Quick Start section with "Show Guide" button', () => {
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        const text = container.textContent ?? '';
        expect(text).toContain('Quick Start');
        const btn = container.querySelector('button.settings-action-btn') as HTMLButtonElement;
        expect(btn).not.toBeNull();
        expect(btn.textContent).toBe('Show Guide');
    });

    it('clicking "Show Guide" calls onClose then onShowOnboarding', () => {
        const onClose = vi.fn();
        const onShowOnboarding = vi.fn();
        const { container } = render(
            <SettingsPanel
                open={true}
                onClose={onClose}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={onShowOnboarding}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        const btn = container.querySelector('button.settings-action-btn') as HTMLButtonElement;
        expect(btn).not.toBeNull();
        fireEvent.click(btn);
        expect(onClose).toHaveBeenCalledOnce();
        expect(onShowOnboarding).toHaveBeenCalledOnce();
    });

    // Phase 50 — Launch Mode section tests (macOS-only)

    it('hides launch-mode section when isMacos=false', () => {
        const { queryByText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        expect(queryByText('Launch Mode (macOS)')).toBeNull();
    });

    it('shows launch-mode radios when isMacos=true', () => {
        const { getByText, getByLabelText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={true}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        expect(getByText('Launch Mode (macOS)')).toBeTruthy();
        expect(getByLabelText('Menu Bar')).toBeTruthy();
        expect(getByLabelText('Window')).toBeTruthy();
    });

    it('calls onLaunchModeChange when Window is selected', () => {
        const onLaunchModeChange = vi.fn();
        const { getByLabelText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={true}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={onLaunchModeChange}
            />
        );
        fireEvent.click(getByLabelText('Window'));
        expect(onLaunchModeChange).toHaveBeenCalledWith('window');
    });

    it('shows restart hint after a launch-mode change', () => {
        const { getByLabelText, getByText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={true}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        fireEvent.click(getByLabelText('Window'));
        expect(getByText('Restart now')).toBeTruthy();
    });

    it('hides the global-shortcut section when isMacos=false', () => {
        const { queryByText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
                globalShortcut="Control+Alt+Command+H"
                onRecordShortcut={vi.fn()}
            />
        );
        expect(queryByText('Global Shortcut (macOS)')).toBeNull();
    });

    it('shows the global-shortcut section with the formatted accelerator when isMacos=true', () => {
        const { getByText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={true}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
                globalShortcut="Control+Alt+Command+H"
                onRecordShortcut={vi.fn()}
            />
        );
        expect(getByText('Global Shortcut (macOS)')).toBeTruthy();
        // Button shows the macOS glyphs for the current accelerator.
        expect(getByText(/Show\/Hide window:\s*⌃⌥⌘H/)).toBeTruthy();
    });

    it('calls onRecordShortcut when the shortcut button is clicked', () => {
        const onRecordShortcut = vi.fn();
        const { getByText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={true}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
                globalShortcut="Control+Alt+Command+H"
                onRecordShortcut={onRecordShortcut}
            />
        );
        fireEvent.click(getByText(/Show\/Hide window:/));
        expect(onRecordShortcut).toHaveBeenCalledTimes(1);
    });
});
