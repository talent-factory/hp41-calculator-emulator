#!/usr/bin/env python3
"""Generate and validate the Tauri-to-SwiftUI feature-parity inventory.

The legacy Tauri sources remain the behavioral reference during migration. This
script extracts their public command, input, state, preference, component, and
platform-event surfaces and joins them to explicit native dispositions.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Iterable


ROOT = Path(__file__).resolve().parent.parent
GUI = ROOT / "hp41-gui"
OUTPUT = ROOT / "docs" / "swiftui-parity.json"
CAPABILITIES = GUI / "swiftui-capabilities.json"
VALID_STATUSES = {"implemented", "partial", "planned", "native-equivalent", "not-applicable"}
COMPLETE_STATUSES = {"implemented", "native-equivalent", "not-applicable"}


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def sorted_unique(values: Iterable[str]) -> list[str]:
    return sorted({value for value in values if value})


def block(text: str, start: str, end: str) -> str:
    start_at = text.index(start)
    end_at = text.index(end, start_at)
    return text[start_at:end_at]


def extract_inventory() -> dict[str, list[str]]:
    lib_rs = read("hp41-gui/src-tauri/src/lib.rs")
    commands_rs = read("hp41-gui/src-tauri/src/commands.rs")
    key_map_rs = read("hp41-gui/hp41-app/src/key_map.rs")
    dispatch_rs = read("hp41-gui/hp41-app/src/dispatch.rs")
    types_rs = read("hp41-gui/hp41-app/src/view.rs")
    prefs_rs = read("hp41-gui/src-tauri/src/prefs.rs")
    app_tsx = read("hp41-gui/src/App.tsx")

    handler = block(lib_rs, "tauri::generate_handler![", "])\n")
    commands = [name for name in re.findall(r"\b[a-z_]+::([a-z_]+)\b", handler) if name != "generate_handler"]

    resolver_match = block(key_map_rs, "match key_id {", "_ => resolve_parameterized(key_id)")
    direct_keys = [
        value
        for value in re.findall(r'"([^"\n]+)"', resolver_match)
        if re.fullmatch(r"[a-z0-9_.]+", value)
    ]
    direct_keys.extend(re.findall(r'key_id == "([^"\n]+)"', commands_rs + dispatch_rs))
    direct_keys.extend([str(number) for number in range(10)] + [".", "e"])

    prefixes = re.findall(r'strip_prefix\("([^"\n]+)"\)', key_map_rs)
    key_patterns = [f"{prefix}{{argument}}" for prefix in prefixes]

    keyboard_tsx = read("hp41-gui/src/Keyboard.tsx")
    keyboard_block = block(keyboard_tsx, "const TOP_ROW", "// Exported so Vitest")
    keyboard_keys = re.findall(r"\bid:\s*'([^']*)'", keyboard_block)

    modal_block = block(app_tsx, "const MODAL_OPENERS", "};\n\nfunction App")
    modal_openers = re.findall(r"^\s{2}([a-z0-9_]+):\s*\(\)", modal_block, re.MULTILINE)

    pending_ts = read("hp41-gui/src/pending_input.ts")
    modal_kinds = re.findall(r"kind:\s*'([^']+)'", pending_ts)
    modal_results = re.findall(r"dispatchId:\s*`([^`]+)`", pending_ts)
    modal_results.extend(re.findall(r"dispatchId:\s*'([^']+)'", pending_ts))

    state_struct = block(types_rs, "pub struct CalcStateView {", "\n}\n\nimpl CalcStateView")
    state_fields = re.findall(r"^\s*pub\s+([a-z0-9_]+):", state_struct, re.MULTILINE)

    prefs_struct = block(prefs_rs, "pub struct GuiPrefs {", "\n}\n\n/// Default theme")
    preferences = re.findall(r"^\s*pub\s+([a-z0-9_]+):", prefs_struct, re.MULTILINE)

    shortcut_function = block(app_tsx, "function resolveKeyId", "\n}\n\n// Phase 26")
    shortcuts = re.findall(r"return\s+'([^']+)'", shortcut_function)
    shortcuts.extend(re.findall(r"'[^']+':\s*'([^']+)'", shortcut_function))

    invoke_pattern = r"invoke(?:<(?:(?!invoke).)*?>)?\s*\(\s*'([^']+)'"
    invokes = re.findall(invoke_pattern, app_tsx, re.DOTALL)
    invokes.extend(re.findall(invoke_pattern, read("hp41-gui/src/SettingsPanel.tsx"), re.DOTALL))

    components: list[str] = []
    for source in (GUI / "src").glob("*.tsx"):
        if source.name.endswith(".test.tsx"):
            continue
        components.extend(re.findall(r"export function\s+([A-Z][A-Za-z0-9_]*)", source.read_text(encoding="utf-8")))

    rust_sources = "\n".join(path.read_text(encoding="utf-8") for path in (GUI / "src-tauri" / "src").glob("*.rs"))
    platform_events = re.findall(r'(?:emit|listen)\(["\']([^"\']+)["\']', rust_sources + "\n" + app_tsx)
    platform_events.extend(re.findall(r"WindowEvent::([A-Za-z0-9_]+)", rust_sources))

    return {
        "tauri_commands": sorted_unique(commands),
        "frontend_invocations": sorted_unique(invokes),
        "direct_key_ids": sorted_unique(direct_keys),
        "parameterized_key_patterns": sorted_unique(key_patterns),
        "keyboard_key_ids": sorted_unique(keyboard_keys),
        "modal_openers": sorted_unique(modal_openers),
        "modal_kinds": sorted_unique(modal_kinds),
        "modal_dispatch_templates": sorted_unique(modal_results),
        "state_fields": sorted_unique(state_fields),
        "preferences": sorted_unique(preferences),
        "physical_shortcut_actions": sorted_unique(shortcuts),
        "ui_components": sorted_unique(components),
        "platform_events": sorted_unique(platform_events),
    }


def load_capabilities() -> dict:
    value = json.loads(CAPABILITIES.read_text(encoding="utf-8"))
    if value.get("schema_version") != 1:
        raise ValueError("swiftui-capabilities.json must have schema_version 1")
    return value


def build_document() -> dict:
    inventory = extract_inventory()
    capabilities = load_capabilities()
    default_status = capabilities.get("default_status", "planned")
    if default_status not in VALID_STATUSES:
        raise ValueError(f"invalid default status: {default_status}")

    overrides = capabilities.get("overrides", {})
    unknown_categories = sorted(set(overrides) - set(inventory))
    if unknown_categories:
        raise ValueError(f"unknown capability categories: {', '.join(unknown_categories)}")

    entries: dict[str, list[dict[str, str]]] = {}
    counts = {status: 0 for status in sorted(VALID_STATUSES)}
    unknown_items: list[str] = []
    for category, items in inventory.items():
        category_overrides = overrides.get(category, {})
        for item in category_overrides:
            if item not in items:
                unknown_items.append(f"{category}:{item}")
        entries[category] = []
        for item in items:
            override = category_overrides.get(item, {})
            status = override.get("status", default_status)
            if status not in VALID_STATUSES:
                raise ValueError(f"invalid status for {category}:{item}: {status}")
            if status in {"native-equivalent", "not-applicable"} and not override.get("note", "").strip():
                raise ValueError(
                    f"{status} capability requires a justification note: {category}:{item}"
                )
            entry = {"id": item, "status": status}
            if override.get("note"):
                entry["note"] = override["note"]
            entries[category].append(entry)
            counts[status] += 1
    if unknown_items:
        raise ValueError(f"capability overrides not found in inventory: {', '.join(unknown_items)}")

    total = sum(counts.values())
    complete = sum(counts[status] for status in COMPLETE_STATUSES)
    return {
        "schema_version": 1,
        "source": "React/Tauri hp41-gui retained as migration reference",
        "native_target": "SwiftUI hp41-gui",
        "summary": {
            "total": total,
            "complete": complete,
            "remaining": total - complete,
            "counts_by_status": counts,
        },
        "categories": entries,
    }


def encoded(document: dict) -> str:
    return json.dumps(document, indent=2, ensure_ascii=False, sort_keys=False) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="fail if the checked-in inventory has drifted")
    parser.add_argument("--require-complete", action="store_true", help="fail unless no partial/planned items remain")
    args = parser.parse_args()

    try:
        document = build_document()
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"swiftui-parity: {error}", file=sys.stderr)
        return 1

    rendered = encoded(document)
    if args.check:
        current = OUTPUT.read_text(encoding="utf-8") if OUTPUT.exists() else ""
        if current != rendered:
            print("swiftui-parity: inventory drifted; run scripts/swiftui-parity.py", file=sys.stderr)
            return 1
    else:
        OUTPUT.write_text(rendered, encoding="utf-8")
        print(f"wrote {OUTPUT.relative_to(ROOT)}")

    summary = document["summary"]
    print(f"SwiftUI parity: {summary['complete']}/{summary['total']} complete; {summary['remaining']} remaining")
    if args.require_complete and summary["remaining"]:
        for category, entries in document["categories"].items():
            gaps = [entry["id"] for entry in entries if entry["status"] not in COMPLETE_STATUSES]
            if gaps:
                print(f"  {category}: {', '.join(gaps)}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
