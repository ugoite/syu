# syu configuration

<!-- FEAT-DOCS-001 -->

`syu` reads `syu.yaml` from the workspace root.

## Minimal configuration

```yaml
version: 0.0.1
spec:
  root: docs/spec
validate:
  default_fix: false
  allow_planned: true
runtimes:
  python:
    command: auto
  node:
    command: auto
```

## Fields

### `version`

The `syu` CLI version that generated the config. `syu init` keeps this aligned
with the running binary. For backwards compatibility, legacy numeric values are
still accepted when reading existing configs.

### `spec.root`

Controls where `syu` reads philosophy, policy, requirements, and features.

Use a relative path for normal workspaces:

```yaml
spec:
  root: docs/spec
```

### `validate.default_fix`

When `true`, `syu validate` behaves as if `--fix` was passed unless the user
explicitly provides `--no-fix`.

### `validate.allow_planned`

Controls whether `planned` requirements and features are allowed.

- `true`: `planned` items are valid, but they must not declare traces yet
- `false`: any `planned` or legacy `planed` status is rejected

### `runtimes.python.command`

Controls which Python executable `syu` uses for Python inspection.

Use `auto` to let `syu` search `python3` and then `python`.

### `runtimes.node.command`

Reserved for runtime-backed Node.js workflows. Today the TypeScript inspector is
bundled, but keeping the runtime configurable now makes future integrations more
predictable.

## CLI precedence

For autofix behavior, CLI flags override config:

1. `--fix`
2. `--no-fix`
3. `validate.default_fix`

`validate.allow_planned` is configuration-only. There is no CLI flag to
override it.

## Recommended practice

- keep `syu.yaml` in the workspace root
- check it into version control
- set `validate.allow_planned: false` once your branch or release line should
  forbid backlog items
- treat runtime overrides as environment-specific, not project-specific, unless
  your team truly needs a pinned executable name
