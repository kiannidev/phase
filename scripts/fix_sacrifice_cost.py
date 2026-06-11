#!/usr/bin/env python3
"""Bulk-convert AbilityCost::Sacrifice struct variants to tuple SacrificeCost form."""

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

SKIP_DIRS = {".git", "target", "node_modules"}


def parse_value(s: str, pos: int) -> tuple[str, int]:
    s = s[pos:].lstrip()
    if not s:
        return "", pos
    if s[0] in "{[(":
        pairs = {"{": "}", "[": "]", "(": ")"}
        open_c, close_c = s[0], pairs[s[0]]
        depth = 0
        for i, c in enumerate(s):
            if c == open_c:
                depth += 1
            elif c == close_c:
                depth -= 1
                if depth == 0:
                    val = s[: i + 1]
                    return val, pos + (len(s) - len(s.lstrip())) + i + 1
        return s, pos
    depth = 0
    for i, c in enumerate(s):
        if c in "{[(":
            depth += 1
        elif c in "}])":
            depth -= 1
        elif c == "," and depth == 0:
            val = s[:i].strip()
            return val, pos + (len(s) - len(s.lstrip())) + i + 1
    val = s.strip().rstrip(",").strip()
    return val, pos + len(s)


def extract_fields(body: str) -> dict[str, str]:
    fields: dict[str, str] = {}
    pos = 0
    body = body.strip()
    while pos < len(body):
        body_slice = body[pos:].lstrip()
        if not body_slice or body_slice.startswith("}"):
            break
        pos += len(body[pos:]) - len(body_slice)
        m = re.match(r"(?:ref\s+)?(\w+)\s*:", body_slice)
        if not m:
            break
        name = m.group(1)
        val, new_pos = parse_value(body_slice, m.end())
        fields[name] = val.strip()
        pos += new_pos
    return fields


def find_variant_block(text: str, variant: str, start: int = 0) -> tuple[int, int, str] | None:
    needle = f"AbilityCost::{variant}"
    idx = text.find(needle, start)
    while idx != -1:
        brace = text.find("{", idx + len(needle))
        if brace == -1:
            return None
        end = find_block_end(text, brace)
        if end != -1:
            return idx, end, text[brace + 1 : end - 1]
        idx = text.find(needle, idx + 1)
    return None


def find_block_end(text: str, start: int) -> int:
    depth = 0
    i = start
    while i < len(text):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return -1


def convert_sacrifice_block(body: str) -> str | None:
    body_stripped = body.strip()
    if ".." in body_stripped and ":" not in body_stripped.replace("..", ""):
        return None
    fields = extract_fields(body)
    if "target" not in fields:
        return None
    target = fields["target"]
    if "requirement" in fields:
        return f"AbilityCost::Sacrifice(SacrificeCost::new({target}, {fields['requirement']}))"
    count = fields.get("count", "1")
    return f"AbilityCost::Sacrifice(SacrificeCost::count({target}, {count}))"


def convert_power_threshold_block(body: str) -> str | None:
    fields = extract_fields(body)
    needed = {"target", "stat", "comparator", "value"}
    if not needed.issubset(fields):
        return None
    return (
        "AbilityCost::Sacrifice(SacrificeCost::new("
        f"{fields['target']}, SacrificeRequirement::Aggregate {{ "
        f"stat: {fields['stat']}, comparator: {fields['comparator']}, value: {fields['value']} "
        "}}))"
    )


def convert_file(path: Path) -> bool:
    text = path.read_text()
    original = text

    # Simple wildcard match-arm fixes
    text = text.replace("AbilityCost::Sacrifice { .. }", "AbilityCost::Sacrifice(_)")
    text = re.sub(
        r"\|\s*AbilityCost::SacrificePowerThreshold\s*\{\s*\.\.\s*\}",
        "",
        text,
    )
    text = text.replace(
        "AbilityCost::SacrificePowerThreshold { .. }",
        "AbilityCost::Sacrifice(_)",
    )

    # Convert struct constructions (innermost first via repeated passes).
    # Skip match-arm destructuring patterns that lack `field:` syntax.
    for _ in range(200):
        changed = False
        for variant, converter in [
            ("SacrificePowerThreshold", convert_power_threshold_block),
            ("Sacrifice", convert_sacrifice_block),
        ]:
            search_from = 0
            while True:
                found = find_variant_block(text, variant, search_from)
                if not found:
                    break
                start, end, body = found
                replacement = converter(body)
                if replacement is None:
                    search_from = end
                    continue
                text = text[:start] + replacement + text[end:]
                changed = True
                search_from = start + len(replacement)
        if not changed:
            break

    if text != original:
        path.write_text(text)
        return True
    return False


def main() -> None:
    changed_files = []
    for path in ROOT.rglob("*.rs"):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        if convert_file(path):
            changed_files.append(path.relative_to(ROOT))
    print(f"Updated {len(changed_files)} files:")
    for p in sorted(changed_files):
        print(f"  {p}")


if __name__ == "__main__":
    main()
