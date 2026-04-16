# go-only example

This example demonstrates a minimal single-language Go workspace.
It contains one philosophy, one policy, one requirement, and one feature -
all mutually linked and traced to a Go test file and a Go implementation file.

## Files

| Path | What it defines |
|------|-----------------|
| `docs/syu/philosophy/foundation.yaml` | `PHIL-GO-001` - the guiding principle |
| `docs/syu/policies/policies.yaml` | `POL-GO-001` linked to `PHIL-GO-001` |
| `docs/syu/requirements/core/go.yaml` | `REQ-GO-001` with a test trace to `go/app_test.go` |
| `docs/syu/features/languages/go.yaml` | `FEAT-GO-001` with an implementation trace to `go/app.go` |
| `go/app_test.go` | Go test file containing the traced test function |
| `go/app.go` | Go source containing the traced implementation function |

## Try it

```bash
cd examples/go-only
syu validate .
syu list requirement
syu show REQ-GO-001
```

A successful `syu validate .` produces output similar to:

```
syu validate passed
workspace: examples/go-only
definitions: philosophies=1 policies=1 requirements=1 features=1
traceability: requirements=2/2 features=2/2
```

## Key things to notice

- **Go tracing** - syu can verify the named Go function symbols in `.go` files.
- **Explicit IDs** - the traced symbols still carry the stable requirement or
  feature ID in nearby Go comments, even though this minimal adapter does not
  yet verify richer `doc_contains` snippets.
- **Current scope** - this example focuses on explicit file/symbol traces for a
  small Go workspace. It is meant to show a runnable baseline, not a complete
  language-specific feature set.
