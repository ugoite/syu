# syu validation report

## Status

- Result: **PASS**
- Workspace: `.`

## Definitions

- Philosophies: 3
- Policies: 8
- Requirements: 24
- Features: 30

## Traceability

- Requirement-to-test traceability: 97/97
- Feature-to-implementation traceability: 112/112

## Issues

No issues found.

## Suggested next actions

- No action needed.


# Coverage by requirement and feature

This report combines Rust line coverage from `cargo llvm-cov` with the current
`syu` requirement/feature trace graph so local reports can inspect coverage in spec terms.

## Requirements

| Requirement | Linked features | Traced test refs | Rust test file coverage | Linked Rust implementation coverage |
| --- | --- | ---: | ---: | ---: |
| REQ-CORE-010 | FEAT-DOCS-001, FEAT-DOCS-002 | 3 | not instrumented | 100.0% (197/197) |
| REQ-CORE-016 | FEAT-SKILLS-001 | 1 | not instrumented | no Rust files |
| REQ-CORE-005 | FEAT-CHECK-001, FEAT-QUALITY-001 | 1 | not instrumented | 99.9% (8144/8156) |
| REQ-CORE-006 | FEAT-QUALITY-001 | 2 | not instrumented | no Rust files |
| REQ-CORE-007 | FEAT-RELEASE-001 | 1 | not instrumented | no Rust files |
| REQ-CORE-008 | FEAT-INSTALL-001 | 1 | not instrumented | no Rust files |
| REQ-CORE-011 | FEAT-CONTRIB-001 | 1 | not instrumented | not instrumented |
| REQ-CORE-012 | FEAT-CONTRIB-001 | 2 | not instrumented | not instrumented |
| REQ-CORE-013 | FEAT-CONTRIB-002, FEAT-CONTRIB-003 | 2 | not instrumented | no Rust files |
| REQ-CORE-014 | FEAT-QUALITY-001 | 1 | not instrumented | no Rust files |
| REQ-CORE-001 | FEAT-CHECK-001 | 13 | 100.0% (5038/5038) | 99.9% (8144/8156) |
| REQ-CORE-002 | FEAT-CHECK-001 | 14 | 99.6% (2959/2971) | 99.9% (8144/8156) |
| REQ-CORE-003 | FEAT-CHECK-001 | 1 | not instrumented | 99.9% (8144/8156) |
| REQ-CORE-004 | FEAT-REPORT-001 | 5 | 100.0% (390/390) | 100.0% (931/931) |
| REQ-CORE-009 | FEAT-INIT-001, FEAT-INIT-002, FEAT-INIT-003, FEAT-INIT-004, FEAT-INIT-005, FEAT-INIT-006, FEAT-INIT-007 | 8 | 100.0% (1681/1681) | 100.0% (8494/8494) |
| REQ-CORE-015 | FEAT-BROWSE-001, FEAT-BROWSE-002 | 4 | 100.0% (756/756) | 100.0% (1704/1704) |
| REQ-CORE-017 | FEAT-APP-001 | 5 | 100.0% (1810/1810) | 100.0% (2115/2115) |
| REQ-CORE-018 | FEAT-LIST-001, FEAT-LIST-002, FEAT-SHOW-001 | 8 | 100.0% (1148/1148) | 100.0% (612/612) |
| REQ-CORE-019 | FEAT-SEARCH-001 | 6 | 100.0% (779/779) | 100.0% (740/740) |
| REQ-CORE-020 | FEAT-ADD-001 | 4 | 100.0% (1309/1309) | 100.0% (1270/1270) |
| REQ-CORE-021 | FEAT-TRACE-001 | 4 | 100.0% (918/918) | 100.0% (879/879) |
| REQ-CORE-022 | FEAT-VSCODE-001 | 2 | not instrumented | no Rust files |
| REQ-CORE-023 | FEAT-RELATE-001 | 4 | 100.0% (1337/1337) | 100.0% (1534/1534) |
| REQ-CORE-024 | FEAT-LOG-001 | 4 | 100.0% (1322/1322) | 100.0% (2036/2036) |

## Features

| Feature | Linked requirements | Implementation refs | Rust implementation files | Rust implementation coverage |
| --- | --- | ---: | ---: | ---: |
| FEAT-APP-001 | REQ-CORE-017 | 10 | 6 | 100.0% (2115/2115) |
| FEAT-BROWSE-001 | REQ-CORE-015 | 4 | 4 | 100.0% (987/987) |
| FEAT-BROWSE-002 | REQ-CORE-015 | 2 | 2 | 100.0% (717/717) |
| FEAT-LIST-001 | REQ-CORE-018 | 1 | 1 | 100.0% (217/217) |
| FEAT-LIST-002 | REQ-CORE-018 | 1 | 1 | 100.0% (217/217) |
| FEAT-SHOW-001 | REQ-CORE-018 | 1 | 1 | 100.0% (178/178) |
| FEAT-RELATE-001 | REQ-CORE-023 | 3 | 3 | 100.0% (1534/1534) |
| FEAT-SEARCH-001 | REQ-CORE-019 | 3 | 3 | 100.0% (740/740) |
| FEAT-LOG-001 | REQ-CORE-024 | 4 | 4 | 100.0% (2036/2036) |
| FEAT-TRACE-001 | REQ-CORE-021 | 2 | 2 | 100.0% (879/879) |
| FEAT-CHECK-001 | REQ-CORE-001, REQ-CORE-002, REQ-CORE-003, REQ-CORE-005 | 12 | 9 | 99.9% (8144/8156) |
| FEAT-INIT-001 | REQ-CORE-009 | 2 | 2 | 100.0% (1477/1477) |
| FEAT-INIT-002 | REQ-CORE-009 | 2 | 2 | 100.0% (1330/1330) |
| FEAT-INIT-003 | REQ-CORE-009 | 2 | 2 | 100.0% (1330/1330) |
| FEAT-INIT-004 | REQ-CORE-009 | 2 | 2 | 100.0% (1330/1330) |
| FEAT-INIT-005 | REQ-CORE-009 | 2 | 2 | 100.0% (1330/1330) |
| FEAT-INIT-006 | REQ-CORE-009 | 2 | 2 | 100.0% (252/252) |
| FEAT-INIT-007 | REQ-CORE-009 | 3 | 3 | 100.0% (1445/1445) |
| FEAT-ADD-001 | REQ-CORE-020 | 2 | 2 | 100.0% (1270/1270) |
| FEAT-REPORT-001 | REQ-CORE-004 | 6 | 4 | 100.0% (931/931) |
| FEAT-VSCODE-001 | REQ-CORE-022 | 3 | 0 | no Rust files |
| FEAT-DOCS-001 | REQ-CORE-010 | 5 | 1 | 100.0% (197/197) |
| FEAT-DOCS-002 | REQ-CORE-010 | 5 | 0 | no Rust files |
| FEAT-SKILLS-001 | REQ-CORE-016 | 3 | 0 | no Rust files |
| FEAT-CONTRIB-001 | REQ-CORE-011, REQ-CORE-012 | 4 | 1 | not instrumented |
| FEAT-CONTRIB-002 | REQ-CORE-013 | 6 | 0 | no Rust files |
| FEAT-CONTRIB-003 | REQ-CORE-013 | 1 | 0 | no Rust files |
| FEAT-QUALITY-001 | REQ-CORE-005, REQ-CORE-006, REQ-CORE-014 | 13 | 0 | no Rust files |
| FEAT-RELEASE-001 | REQ-CORE-007 | 5 | 0 | no Rust files |
| FEAT-INSTALL-001 | REQ-CORE-008 | 1 | 0 | no Rust files |
