# Spec anti-patterns and refactoring triggers

`syu validate .` can be green while the spec is still drifting toward a shape
that will be painful to maintain. This guide covers the most common
four-layer mistakes that stay technically valid, why they hurt later, and what
kind of refactor usually fixes them.

Use this page when the repository can still validate, but the spec already
feels noisy, repetitive, or hard to explain to another contributor.

## Philosophy anti-patterns

### Philosophy that changes every sprint

**Smell:** a philosophy entry changes whenever the roadmap changes.

```yaml
- id: PHIL-001
  title: Ship the Q3 login redesign
```

That is not a stable philosophy. It is a short-lived project goal.

**Why it hurts:** philosophy should survive multiple features and releases. When
it changes every sprint, the rest of the graph loses its long-term anchor.

**Refactor signal:** if the statement only matters for one initiative or one
release window, move it down to a requirement or feature. Keep philosophy for
the ideals and trade-offs that still matter after the current work lands.

### Philosophy that names tools, files, or frameworks

**Smell:** philosophy says which framework to use or which file layout to keep.

That usually belongs in policy or a concrete requirement.

**Refactor signal:** if the statement starts sounding like a repository rule
contributors must follow, promote it to policy. If it only exists to support
one behavior, move it down to a requirement.

## Policy anti-patterns

### Policy that just repeats one requirement

**Smell:** the policy and requirement say almost the same thing.

```yaml
- id: POL-AUDIT-001
  title: Every login response must include an audit ID.
- id: REQ-AUDIT-001
  title: Every login response must include an audit ID.
```

**Why it hurts:** policy should be the reusable rule; requirement should be one
concrete obligation that satisfies that rule. If both layers say the same
thing, contributors stop learning anything from the extra indirection.

**Refactor signal:** if the policy only applies to one requirement, fold the
text into that requirement or broaden the policy so multiple requirements can
inherit it.

### One policy per feature

**Smell:** every feature gets its own private policy.

**Why it hurts:** policy becomes decorative instead of repository-wide guidance.
The graph is still valid, but the layer stops helping contributors understand
what rules are shared.

**Refactor signal:** merge feature-specific policies into a smaller set of rules
that actually span multiple requirements or contributor workflows.

## Requirement anti-patterns

### Requirement that is really an implementation note

**Smell:** the requirement says *how* to code something instead of *what must be
true*.

```yaml
- id: REQ-LOGIN-001
  title: Call `issue_session_cookie()` before `render_dashboard()`
```

That is a code note, not a requirement.

**Why it hurts:** the requirement becomes brittle as soon as the implementation
changes. Reviewers can no longer tell whether the repository still satisfies the
real obligation after a refactor.

**Refactor signal:** rewrite the requirement in outcome language, then keep the
implementation detail in the feature trace or code itself.

### Requirement that tries to ship a whole roadmap

**Smell:** one requirement collects several unrelated behaviors because they all
belong to the same project area.

**Why it hurts:** delivery status, tests, and maintenance become impossible to
read. A single `implemented` status can hide partial delivery, and one broken
trace can blur unrelated work.

**Refactor signal:** split when one requirement needs different tests, different
owners, or a different delivery cadence from the rest.

## Feature anti-patterns

### Feature named after a file instead of behavior

**Smell:** the feature title is basically the file path or module name.

```yaml
- id: FEAT-LOGIN-001
  title: Update `src/login.rs`
```

**Why it hurts:** file names change more often than user-visible behavior. A
feature should still make sense after code moves to a different module.

**Refactor signal:** rename the feature around the capability it delivers, then
keep the file path in the implementation traces.

### Umbrella feature that hides unrelated work

**Smell:** one feature points to many symbols because they all live in the same
area, but they do not really implement one coherent behavior.

**Why it hurts:** maintenance becomes fuzzy. Contributors can no longer tell
which symbol change belongs to which behavior, and traceability turns into a
catch-all bucket.

**Refactor signal:** split when a reviewer would reasonably ask for different
titles, summaries, or requirements for the traced symbols.

## Document-level smells

### One document keeps growing forever

**Smell:** a requirement or feature file validates, but it is now a dumping
ground for multiple areas, contributors, or delivery phases.

**Refactor signal:** split the file when people start scrolling past unrelated
entries to find the one they want. `syu` is happier with several focused
documents than one technically valid but hard-to-navigate blob.

### Folder names no longer match the scope

**Smell:** entries were moved between domains, but the file names and folder
layout still reflect the old structure.

**Refactor signal:** rename or move documents when the path stops helping a new
contributor guess where to add the next related item.

## When to merge, split, promote, demote, or rename

- **Split** when one item needs multiple owners, test surfaces, or delivery
  cadences.
- **Merge** when two neighboring items always change together and cannot be
  explained separately.
- **Promote** a statement upward when it has become a shared rule or enduring
  principle instead of one narrow obligation.
- **Demote** a statement downward when it only exists to support one specific
  feature or implementation path.
- **Rename** when the current title only makes sense because of an old file
  layout, project codename, or temporary initiative.

## Practical refactoring loop

1. Use `syu show <ID> .` to inspect the current item and its links.
2. Rewrite or split the spec item first.
3. Update reciprocal links immediately so the graph stays navigable.
4. Run `syu validate .` after each small refactor instead of waiting for one
   large rewrite.
5. If the refactor is about user-visible behavior rather than validation errors,
   use the [tutorial](./tutorial.md) and [troubleshooting guide](./troubleshooting.md)
   together: the tutorial shows healthy shapes, and troubleshooting explains the
   concrete failures you might trigger while cleaning up.
