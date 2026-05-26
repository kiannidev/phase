---
name: bug-coverage-classifier
description: Use when classifying a Phase bug report against a specific card's engine-authoritative parse_details to decide whether the misbehaving aspect is in a supported clause (real defect — investigate) or unsupported clause (known coverage gap — defer unless trivial). Reads the same coverage data the Alt-hover overlay and card-bot render. Invoked by /bug-triage and /issue-clusterer.
---

# Bug Coverage Classifier

## Inputs

- `card_name` — the card the bug report names (e.g. `"Lurrus of the Dream-Den"`).
- `bug_description` — what the user said is misbehaving.

## What to do

### 1. Look up the card

Fetch the coverage data and extract just the entry for this card:

```bash
curl -s "https://pub-fc5b5c2c6e774356ae3e730bb0326394.r2.dev/preview/coverage-data.json" \
  | jq --arg n "<card_name>" '.cards[] | select(.card_name == $n) | {oracle_text, supported, parse_details}'
```

If `jq` returns nothing, try common name normalizations: drop apostrophes (`Bloodchief's Ascension` → `Bloodchief Ascension`), try the front face of an MDFC, check spelling. If the card still cannot be found, consult `triage/unknown-card-mapping.json` for known corrections. If the name is flagged `not_a_card` (e.g. token types like "Blood Token"), return verdict `not_card_data_attributable`.

For repeat lookups in the same session, cache the full coverage JSON to a tmp file once and `jq` against the file.

### 2. Read the parse_details tree

Each node has:
- `category` — `ability`, `triggered_ability`, `static`, `keyword`, etc.
- `label` — engine type tag (e.g. `DealDamage`, `GraveyardCastPermission(Cast,once_per_turn)`).
- `source_text` — the Oracle text fragment this node represents.
- `supported: true | false` — whether the engine claims to handle this clause.
- `details` — typed sub-data (effect parameters, targets, conditions).

### 3. Identify which node the bug is about

Read the bug description and match it to the node whose `source_text` describes the misbehaving line. Use your judgement — bug reports use synonyms, abbreviations, and concrete card-game examples (`"X=3"`, `"on end step"`, `"with 2 counters"`) that won't always share vocabulary with the Oracle text. The card name is already pinned, so you're choosing among at most a handful of clauses.

If the card has only one parse_details node, that's the node — unambiguous.

If you genuinely cannot decide which node the bug is about, return verdict `cannot_determine` and ask the reporter (or the calling skill) for a quote of the misbehaving Oracle line.

If the bug isn't about card text at all (combat assignment, AI behavior, UI rendering, multiplayer sync, deckbuilder, etc.), return verdict `not_card_data_attributable`.

### 4. Apply the rubric

| Matched node | Verdict | Triage signal |
|---|---|---|
| `supported: true` | `supported_aspect_defect` | The engine claims this works. Bug is either a parser misparse (AST wrong) or a runtime bug (handler wrong). Investigate. |
| `supported: false` | `unsupported_aspect` | Known coverage gap. Defer unless the fix is trivial. |
| — (no card match, or off-card concern) | `not_card_data_attributable` | Not a parser/effect-handler concern. Investigate the relevant subsystem (combat/AI/UI/MP). |
| — (ambiguous) | `cannot_determine` | Need more info from the reporter. |

## Important caveats

- **`supported: true` does not prove the AST is correct.** A clause can parse to the wrong AST shape and still be marked supported because the parser produced *some* typed node. See project memory `project_backlog_is_parser_misparses.md`.
- **`supported: true` does not prove the runtime consumes the AST.** A parsed field can be a silent no-op if no handler reads it. See `project_parsed_ast_not_consumed.md`.
- Both of those mean `supported_aspect_defect` is the *correct* verdict for a misbehaving supported clause even though the underlying root cause might be parser-level rather than runtime-level — distinguishing those is downstream investigation work, not classification work.

## Output shape

Return a single object the calling skill can use:

```json
{
  "card_name": "<as input>",
  "verdict": "supported_aspect_defect | unsupported_aspect | not_card_data_attributable | cannot_determine",
  "matched_clause": {
    "label": "<engine label>",
    "source_text": "<Oracle fragment>",
    "supported": true | false
  } | null,
  "reasoning": "<one or two sentences citing which node matched and why>"
}
```

## When you are the caller

If you're invoking this skill from `/bug-triage` or `/issue-clusterer`, the calling instructions should pass `(card_name, bug_description)` per report and use the verdict to inform the NEW / DUP / APPEND / HANDLED decision — not to gate it. Maintainer review is still the final arbiter.
