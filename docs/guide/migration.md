# Migration guide

This page documents breaking changes and upgrade steps for each `syu` alpha
release. When `syu validate .` starts failing after an upgrade, check the
section for the version you just installed.

> **Note:** `syu` is in alpha. The config schema, YAML spec format, and CLI
> flags may change in any alpha release. This guide is updated with every
> release that introduces user-visible breaks.

---

## Upgrading to `v0.0.1-alpha.7`

### New `syu.yaml` fields

| Field | Default | Notes |
|---|---|---|
| `validate.require_reciprocal_links` | `true` | New. Adjacent-layer links must be reciprocal. See [SYU-graph-reciprocal-001](./troubleshooting.md#syu-graph-reciprocal-001--missing-back-link). |
| `report.output` | *(none)* | New. Sets a default output path for `syu report`. Previously `--output` was always required. |

### Action required

**`validate.require_reciprocal_links: true` is on by default.**

If your spec has one-directional links (e.g. a requirement lists a feature but
the feature does not list the requirement back), `syu validate` will now fail
with `SYU-graph-reciprocal-001`.

To fix: add the missing back-references. Or, if your project needs a phased
migration, temporarily opt out:

```yaml
# syu.yaml
validate:
  require_reciprocal_links: false
```

### New validation rules enabled by default

| Code | Severity | Description |
|---|---|---|
| `SYU-graph-reciprocal-001` | error | Adjacent-layer links must be reciprocal |

### Spec root default

The `spec.root` default remains `docs/syu` (unchanged from alpha.6).

---

## Upgrading to `v0.0.1-alpha.6`

### `spec.root` default changed

| | alpha.5 | alpha.6 |
|---|---|---|
| Default `spec.root` | `docs/spec` | `docs/syu` |

If your `syu.yaml` relied on the implicit default (`docs/spec`) without
explicitly declaring `spec.root`, you must either:

1. Move your spec directory: `mv docs/spec docs/syu`, or
2. Add `spec: root: docs/spec` to `syu.yaml` to keep the old path.

### New structured validation rule IDs

alpha.6 introduced the `SYU-*` rule code taxonomy. Error output format changed:

| Before (alpha.5) | After (alpha.6) |
|---|---|
| `error: orphaned item REQ-001` | `error[SYU-graph-orphaned-001] REQ-001` |

CI scripts that grep for the old error format may need updating.

---

## Upgrading to `v0.0.1-alpha.5`

### Hierarchical folder support

alpha.5 added support for nested `features/` and `requirements/` directories.
Flat single-file layouts (`features.yaml`, `requirements.yaml`) continue to
work; no migration is required.

### Runtime auto-detection

The `runtimes.python.command` and `runtimes.node.command` fields now accept
`auto` (default) in addition to explicit paths. If you previously hard-coded
the interpreter path, `auto` will use whatever is on `$PATH`, which may pick
up a different version in CI.

---

## Contributing migration notes

Every PR that introduces a **breaking change** — to `syu.yaml` fields, spec
YAML schema, CLI flag names, or default validation behaviour — **must** add an
entry to this file before merge. The entry should include:

1. The version being changed (use the next planned alpha tag)
2. A table showing old → new for any config/schema changes
3. The exact action required to upgrade an existing repository
4. Any new default-on validation rules

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for the full contribution workflow.

---

## Version compatibility summary

| syu version | `spec.root` default | `require_reciprocal_links` | `report.output` |
|---|---|---|---|
| alpha.1–alpha.4 | — (not yet documented) | — | — |
| alpha.5 | `docs/spec` | not present | not present |
| alpha.6 | `docs/syu` | not present | not present |
| alpha.7 | `docs/syu` | `true` | `null` |
