# Maintainer playbook: merge queue required checks

<!-- FEAT-CONTRIB-002 -->

When maintainers intentionally rename queue-gated jobs or add/remove
`merge_group` workflows, update `.github/merge-queue-checks.json` alongside the
workflow files.

That manifest is the checked-in source of truth for the required merge-queue
check names this repository expects to protect `main`.
