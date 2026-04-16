# Trace adapter capability matrix

When you choose a `tests:` or `implementations:` language key, three validator
capabilities matter most:

- **symbol existence validation** — whether `syu validate` can confirm that the
  declared symbol is really present in the traced file
- **`doc_contains` validation** — whether `syu` can inspect symbol
  documentation comments and enforce required snippets
- **strict `validate.require_symbol_trace_coverage` inventory** — whether `syu`
  can scan the repository for unowned public APIs and tests in that language

This page summarizes the built-in trace adapters shipped on `origin/main`
_today_. Use it before you turn on `doc_contains` checks widely or enable strict
coverage in a mixed-language repository.

## Built-in adapter matrix

| Built-in adapter | Accepted `lang:` aliases / files | Symbol existence validation | `doc_contains` validation | Strict `require_symbol_trace_coverage` inventory |
| --- | --- | --- | --- | --- |
| Rust | `rust`, `rs` / `.rs` | ✅ Rich symbol inspection plus declaration matching | ✅ | ✅ |
| Python | `python`, `py`, `pytest`, `unittest` / `.py` | ✅ Rich symbol inspection plus pattern fallback | ✅ | ✅ |
| TypeScript / JavaScript | `typescript`, `ts`, `tsx`, `javascript`, `js`, `jsx`, `vitest`, `bun`, `bun-test` / `.ts`, `.tsx`, `.js`, `.jsx` | ✅ Rich symbol inspection plus pattern fallback | ✅ | ✅ |
| Shell | `shell`, `sh`, `bash`, `zsh` / `.sh`, `.bash`, `.zsh` | ✅ Pattern-based symbol matching | ❌ | ❌ |
| YAML | `yaml`, `yml` / `.yaml`, `.yml` | ✅ Pattern-based symbol matching | ❌ | ❌ |
| JSON | `json` / `.json` | ✅ Pattern-based symbol matching | ❌ | ❌ |
| Markdown | `markdown`, `md` / `.md` | ✅ Pattern-based symbol matching | ❌ | ❌ |
| Gitignore | `gitignore`, `ignore` / `.gitignore` | ✅ Filename-aware, pattern-based matching | ❌ | ❌ |

## What the strict inventory scans today

When `validate.require_symbol_trace_coverage: true` is enabled, `syu` currently
builds ownership inventories for these languages only:

- **Rust** — public items in `src/`, plus `#[test]` functions in `src/` and
  `tests/`
- **Python** — public names in `src/` that do not start with `_`, plus
  `test_...` and `Test...` symbols in `tests/`
- **TypeScript / JavaScript** — exported symbols in `src/`, plus `test...` and
  `Test...` symbols in `tests/`

Other built-in adapters still validate the traces you declare explicitly, but
they do **not** participate in the repository-wide strict ownership scan.

## How to choose the right promises

- Need `doc_contains`? Use Rust, Python, or TypeScript / JavaScript traces.
- Need strict ownership coverage? Keep it limited to Rust, Python, or
  TypeScript / JavaScript until another language gains an inventory scanner.
- Using Shell, YAML, JSON, Markdown, or Gitignore traces? Keep the mapping to
  `file` + `symbols` (or `symbols: ["*"]` when one file intentionally belongs
  to one item), but do not expect doc-comment inspection or strict ownership
  inventory.

Even in rich-inspection languages, wildcard traces cannot use `doc_contains`
because `symbols: ["*"]` does not point to one inspectable symbol.

## What about Go?

`lang: go` is **not** a built-in trace adapter on `origin/main` today, so
`syu validate` reports `SYU-trace-language-001` instead of treating Go as a
supported symbol-validation target. Re-check the implementation before relying
on Go-specific `doc_contains` or strict coverage behavior in future branches.
