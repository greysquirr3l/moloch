# Plan: make Moloch better at improving the workflow it generates

Status: proposal. Not started.

Inputs: `project-docs/WORKFLOW_IMPROVEMENTS.md` (the "WI doc") and two articles
by Jose Crespo (AI Advances): "LLMs Are the Worst Kind of AI Right Now. Here's
Their Replacement" (Nov 2026) and "Geo-AI Will Take Your Job. Manifold
Programming Saves It" (Jun 2026).

## What this plan is about

The WI doc makes the **generated** scaffold safer: verify commands, stop
conditions, baselines, evaluator independence. This plan covers the next
layer: Moloch's **improvement loop**, meaning how it learns from a run and
changes the scaffold, the process docs and the prompts it emits.

Today that loop is `retro`, `replan` and `patterns`, plus `check` and
`validate --lint` as quality gates. Reading the code shows it is the weakest
part of the tool.

## What the article gives us (and what it doesn't)

Both articles are vision pieces. "Transportive AI" (category-theory contexts,
functors, profunctors) and "manifold programming" (metric, transport, holonomy)
have no implementation, benchmark or evidence behind them.
**Do not build any of that.** It also contradicts the WI doc's own constraint
that Moloch stays a scaffolder with no LLM execution.

Six practical ideas survive once the maths is stripped out. Each maps to a
concrete change below.

| Article idea | Plain meaning | Applies to Moloch as |
|---|---|---|
| Contexts are nested; an approval in one must not leak to another | An "approved" in staging is not approval for production | Scope every approval and gate (Phase 3) |
| Declared laws | State what must stay true as checkable rules | Plan-level invariants with a check command (Phase 4) |
| Loop check / drift | Go around a cycle and measure what changed | Measure drift across generate, run, retro, regenerate (Phase 5) |
| The model proposes, a separate map decides | Don't let the writer approve its own work | Typed, deterministic improvement patches that Moloch applies and checks (Phases 1 and 2) |
| Never trust one assistant; a second model checks (Geo-AI) | The checker must be a different model from the writer | Lint `evaluator-not-independent` (WI W5, task T10) |
| The confusion matrix: when the AI said yes, was it good? | Measure the checker's false positives and misses, not just whether it passed | Evaluator confusion matrix in `retro --compare` (Phase 5) |
| FPGA vs ASIC (Geo-AI) | Flexible reusable fabric vs one frozen design per job | Generic tasks vs hardened task kinds; promote what recurs (Phase 7) |

## Gaps found in the current improvement loop

Verified by reading `src/adapters/retro.rs`, `replan.rs`, `patterns.rs` and
`cli.rs`:

1. **Keyword matching on prose.** `retro` lowercases the free-text learnings
   column of `PROGRESS.md` and tests `contains("fix")`, `"lint"`, `"failed"`,
   and so on. "fix" matches almost any learning. Suggestions are unranked and
   not grounded in counts.
2. **Suggestions can only target the plan.** `retro --apply` writes to
   `plan.toml`. Nothing in the loop proposes a change to the orchestrator
   prompt, the evaluator prompt, `PROGRESS.md` structure or the strict rules,
   which are the process docs.
3. **Suggestions are strings.** `RetroSuggestion` has `pattern`, `suggestion`
   and `task_slug`. A saved pattern in `~/.moloch/patterns/` is a list of
   strings, so it cannot be re-applied deterministically to a new plan.
4. **Input is unstructured.** The scaffold tells the orchestrator to append
   "learnings" as prose. There is no failure category, no environment, no
   retry count field, so there is little for `retro` to count.
5. **No before/after measure.** Nothing shows an applied improvement helped.
   `check` scores a plan, not a run.
6. **Approvals have no scope.** Gates are free text, and approvals recorded in
   `PROGRESS.md` carry no task, environment or expiry. This is the article's
   "approved in staging, applied to production" failure.

## Evidence from real runs

`~/Projects/rust` holds 24 `PROGRESS.md` files (about 2.9 MB; 22 of them in
the Moloch shape, the rest scratch copies or free-form). On 2026-11-20 I ran
the current `moloch retro` over the 22 outside this repo. Read-only; nothing
was applied or written.

- **Retry counts are wrong.** `retro` reported 95 retries for muss (37 tasks),
  171 for obliviate (181 tasks) and 89 for reudora (53 tasks). It counts any
  task-result cell containing "fix", "failed", "attempt" or "retry", so a
  description like "fix the lockfile" counts as a retry.
- **Output is boilerplate.** 839 suggestions across 22 files, about 38 each.
  muss gets "T03 hit a blocking gate" for six tasks that merely mention a
  gate or "blocked" in prose, plus the same `lint-config` advice repeatedly.
- **It reads the wrong place.** The parser reads the third column of the task
  table, which in real files is a result summary. The actual lessons live in
  the `## Accumulated Learnings` bullet list (22 files) and in per-task
  headings.
- **The best signal is ignored.** Several files record evaluator verdicts as
  headings with scores and retry counts, for example "T25 evaluator FAIL
  (about 6.8/10) on `594ac77` — retry 1 of 2", then "re-evaluation FAIL",
  then "final re-evaluation PASS (about 9.3/10)". `retro` does not read any
  of it, though it is exactly the structured failure evidence Phase 1 asks
  for.
- **Parsing edge cases.** `scout` has no `| T..` rows (0 tasks) and one file
  yields the project name "Progress".
- **Many tasks are marked blocked at some point** (muss 15 mentions,
  protocol-droid 14), which makes "blocked" a common word, not a signal.

Consequence: the corpus is the test bed for every phase, and Phase 0 below
builds it first.

## Prerequisites from the WI doc

This plan depends on WI items W0, W3, W4, W6 and W7 (baseline, `verify`,
`stop_when`, baseline-aware preflight, golden corpus). Do those first. W5
(evaluator independence) is small and independent; it is task T10 in
`plan.toml`. In
particular:

- `verify` gives `retro` an objective failure signal (which command failed).
- The golden corpus (W7) is the regression harness for every phase here.
- Baseline-aware preflight (W6) stops pre-existing failures being counted as
  task failures.

One baseline note: `cargo run` on a clean `main` failed today with
`warnings are denied by build.warnings` for an unused `anyhow` dependency
(`Cargo.toml:82`). W0 must record this before any edit; it is not ours to
hide inside a feature branch.

## Constraints (inherited from the WI doc)

- Moloch stays a scaffolder: no LLM calls, no process management, no commits.
- Additive TOML only, `#[serde(default)]` on every new field.
- No new dependencies without asking.
- Generated prompts must work for small models: checklists, explicit commands.
- Any text Moloch did not write (a `PROGRESS.md` learning is written by an
  agent, so it counts) is treated as data, not instructions, when it is
  quoted into another prompt. Reuse the WI doc's W2 wrapper.

## Phases

Each phase ships on its own and ends with its Verify step.

### Phase 0: real-run corpus

Do this before Phase 1; it is the ground truth for tuning.

- Copy a sample of the real `PROGRESS.md` files into `tests/progress-corpus/`.
  Choose for spread: small and large, with and without evaluator verdicts,
  and the odd shapes (`scout`, a scratch copy). Aim for 8 to 10 files.
- **Ask the owner first.** These are other projects, and some name
  security-sensitive work (`securemessages-rs`, `protocol-droid`). Redact
  names, URLs, commit hashes and any secret-looking strings, or write
  synthetic files from the real shapes. Never commit unredacted copies.
- Record today's `retro` output per file as a snapshot, so each later phase
  shows what changed.
- Hand-label a few files: for each, the real retries, the real blocking
  gates, and the lessons a good retro would surface. That label set is what
  Phase 2 is scored against (precision and recall, not "feels better").

**Done:** corpus, snapshots and labels committed, redacted.
**Verify:** a test parses every corpus file without panic and asserts task
counts against the labels.

### Phase 1: structured run evidence

Fix gap 4 first; everything else needs countable input.

- Extend the `PROGRESS.md` and `RUN_LOG.md` templates so each retry or failure
  is one row with fixed fields: task, attempt number, failing command, category
  (`spec-gap`, `env`, `scope`, `tooling`, `baseline`, `approval`, `other`),
  one-line cause.
- Keep the existing prose learnings column. Add the structured table beside
  it; do not replace it.
- Teach `domain/report.rs` `parse_progress` to read the table. Old files
  without it parse exactly as before.
- Also parse what real runs already contain: evaluator verdict headings
  (verdict, score, retry n of m) and the `## Accumulated Learnings` bullets,
  so existing projects benefit without regenerating anything.

**Done:** a generated `PROGRESS.md` has the failure table; `parse_progress`
returns typed rows; legacy files still parse.
**Verify:** unit tests for both formats; golden corpus still passes;
`cargo test --all`.

### Phase 2: typed improvement patches

Fixes gaps 1, 2 and 3.

- Replace `RetroSuggestion { suggestion: String }` with a typed suggestion:
  `{ evidence, target, patch, severity }`.
  - `target`: `Task(slug)`, `Orchestrator`, `Evaluator`, `Rules`, `Plan`.
  - `patch`: a small enum of operations that map to existing plan fields, for
    example `AddHint`, `AddVerify`, `AddStopWhen`, `AddRule`, `AddGate`,
    `SplitTask`, `AddDependency`.
- Process-doc improvements flow through plan fields (`orchestrator.rules`,
  `stop_when`, `verify`), not by editing generated markdown. That keeps the
  scaffold regenerable and the TOML additive.
- Derive suggestions from counts over the Phase 1 table and the parsed
  evaluator verdicts, not substrings. Retries come from verdict headings and
  failure rows, never from words in a description. Each
  rule states its threshold in code (for example: two or more `env` failures
  across tasks means "add an env-setup task or preflight step").
- `retro --apply` applies patches to `plan.toml`. `retro --json` emits them.
- `patterns` stores patches, so a saved pattern re-applies to a new plan.

**Done:** every suggestion has evidence rows and a patch; applying a patch
changes `plan.toml` and nothing else; saved patterns round-trip.
**Verify:** table-driven unit tests, one per rule, plus the Phase 0 labelled
corpus: report precision and recall against the labels, and the suggestion
count per file (target: a handful, not about 38). Retry counts must match
the labels.

### Phase 3: scoped approvals (the article's one actionable idea)

Fixes gap 6. Template-first, with one small schema addition.

- Add an **Approvals** table to the `PROGRESS.md` template: scope (task and
  environment), action, approved by, expires at (default: end of that task).
- Add to the orchestrator and subagent prompts: an approval applies only to its
  recorded scope; an approval for a test or staging action never authorizes a
  production action; when scope is unclear, stop and ask (links to WI W4).
- Optional `scope = "..."` on a task gate so the gate text names what is being
  approved. Add lint `gate-without-scope` (Info).

**Done:** generated prompts contain the scope rule and the table; a gate can
carry a scope.
**Verify:** integration test on the strings; lint unit test; fixtures still
validate.

### Phase 4: declared invariants

Plan-wide laws that outlive any one task.

- Add `[[invariants]]` to the plan: `name`, `check` (a command that exits
  zero), optional `applies_to` (task slugs, default all).
- Render a **Laws** section in the orchestrator: run every applicable
  invariant at each task boundary and once more at the end of the plan.
- `retro` adds an invariant suggestion when the same regression recurs
  (Phase 2 rule).
- Lint `invariant-no-check` (Warning) for an invariant with an empty command.

This overlaps W3 `verify`. The split: `verify` is per task, `invariants` are
plan-wide and re-run at the end.

**Done:** invariants render, run at the boundary and at the end.
**Verify:** unit and integration tests; golden plans gain one invariant.

### Phase 5: measure drift between runs

Fixes gap 5, using the article's loop check as a plain metric.

- Add `moloch retro --compare <old-progress> <new-progress>` reporting retries
  per task, gate waits, failures by category and baseline failures.
- Add a regeneration check: `generate` from `plan.toml`, apply the patches,
  `generate` again. The second result must pass `check` with a score no lower
  than the first, and `validate --lint` must report no new diagnostics. Run it
  in the golden corpus test.
- Add a `replan` idempotence test: re-rendering a task with no new evidence
  gives byte-identical output.
- Add an **evaluator confusion matrix** to `retro --compare`, built from the
  parsed verdict headings (Phase 1):
  - false positive: an evaluator PASS on a task that is later reopened,
    retried or followed by a fix commit for that task;
  - false negative: an evaluator FAIL later overturned by a PASS with no
    relevant code change between the two verdicts;
  - missing evidence counts as "unknown", never as a hit;
  - report counts and the two rates, and assert them against hand labels on
    two or three corpus files.

**Done:** compare command (with the evaluator matrix) and the two tests exist.
**Verify:** `cargo test --all`; break a patch on purpose and confirm the
regeneration check fails.

### Phase 6 (optional): improvement brief for the user's agent

Moloch cannot call an LLM, but it can hand the user's agent a good prompt.

- `moloch retro --brief` writes `IMPROVEMENT_BRIEF.md` in the task-spec format
  from the WI doc: goal, the evidence table, the typed patches as proposals,
  constraints, a Definition of done and a Verify step.
- Quoted learnings are wrapped as untrusted data with a per-build random id
  (WI W2).

**Done:** brief generated and validated by a unit test on structure.
**Verify:** the golden corpus asserts the brief has every required section.

### Phase 7 (later, gated): promote recurring patterns to hardened kinds

The Geo-AI article's FPGA/ASIC analogy applied to tasks: a generic task is the
flexible fabric; a hardened `TaskKind` is the frozen design (fixed checklist,
fixed `verify` skeleton, mandatory gate, golden test).

- Add a `PromoteToKind` retro suggestion that fires when the same failure
  category and task shape recur across several corpus projects. Candidate
  shapes: migrations, dependency bumps, security audits, docs-only changes.
- Suggestion only. Moloch never adds a kind by itself; a human decides.
- Human gate: if no pattern recurs in at least three projects, report that and
  skip implementation.

**Done:** the patch variant and rule exist, silent on single files, firing on
the aggregate.
**Verify:** unit test on labelled corpus files; `cargo test --all`.

## Order and size

| Phase | Depends on | Rough size | Touches |
|---|---|---|---|
| 0 | owner OK to copy | small | `tests/progress-corpus/` |
| 1 | 0, WI W0, W7 | small | templates, `report.rs` |
| 2 | 1 | large | `retro.rs`, `patterns.rs`, `cli.rs` |
| 3 | WI W4 | small | templates, `plan.rs`, `lint.rs` |
| 4 | WI W3 | medium | `plan.rs`, templates, `lint.rs` |
| 5 | 2 | medium | `retro.rs`, tests |
| 6 | 2, WI W2 | small | `retro.rs`, template |
| 7 | 2, 5 | small | `retro.rs` |

Phases 0 to 3 give most of the value. Stop after 3 if time is short.

## Risks and open questions

- **Thresholds still need tuning.** The corpus fixes the "no real runs"
  problem, but 22 files from one author's projects is a small, biased sample.
  Say in the code that thresholds were tuned on it.
- **Privacy.** See Phase 0: redact or synthesize before anything is committed.
- **Agents may not fill the structured table.** Small models skip formats.
  Keep the prose column as the fallback and have `retro` report how many
  failures had no structured row.
- **Patch vocabulary creep.** Keep the `patch` enum to operations that map to
  existing plan fields. A patch that needs a new field is a design decision
  for the human.
- **Decision for the owner:** should `invariants` replace `[preflight]` over
  time, or stay separate? This plan keeps them separate.

## What this plan does not do

- No LLM calls, no embeddings, no semantic matching of learnings.
- No category-theory or geometry machinery (metrics, curvature, holonomy
  angles); "drift" means the counts above.
- No change to existing TOML field meanings.
