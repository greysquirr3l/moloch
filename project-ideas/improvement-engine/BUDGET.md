# BUDGET — moloch-improvement-engine

Token estimates for the generated scaffold. Use this file to budget per-iteration prompt sizes before running the orchestrator loop.

## Per-task context size

| # | Task | Tokens | Status |
|---|------|-------:|--------|
| T01 | task-verify-commands | 302 | ok |
| T02 | stop-and-baseline | 286 | ok |
| T03 | golden-plan-corpus | 203 | ok |
| T04 | progress-corpus | 297 | ok |
| T05 | structured-failure-rows | 213 | ok |
| T06 | typed-patches | 305 | ok |
| T07 | scoped-approvals | 181 | ok |
| T08 | plan-invariants | 168 | ok |
| T09 | drift-compare | 343 | ok |
| T10 | evaluator-independence | 250 | ok |
| T11 | promote-to-kind | 266 | ok |
| T12 | integration-wiring | 434 | ok |
| T13 | stub-cleanup | 340 | ok |

## Summary

| Metric | Tokens |
|--------|-------:|
| Total task contexts | 3588 |
| Plan + progress + orchestrator base overhead | 1006 |
| **Total estimated tokens** | **4594** |
| Largest single task (integration-wiring) | 434 |

## Thresholds

Tokens are measured against the cl100k_base-style heuristic used by `moloch check`. Tasks above the WARN threshold risk losing detail to context window churn; tasks above CRITICAL will almost certainly truncate.

| Threshold | Tokens |
|-----------|-------:|
| WARN threshold | 80000 |
| CRITICAL threshold | 150000 |

## Recommended daily cap

**Recommended daily cap: 4340 tokens** (largest task × 10, so the loop can complete roughly 10 iterations before context churn forces a fresh subagent). Adjust downward if your orchestrator model has a smaller context window.

## Cost tiers

Per-iteration cost under three operating modes. Use this to plan session budgets before running the orchestrator.

| Mode | Tokens | % of total |
|------|-------:|-----------:|
| noop   | 200  | 4% |
| report | 1531 | 33% |
| action | 9188 | 200% |

Stable fraction between iterations: **0.4** (estimated reuse of PROGRESS.md + accumulated learnings; tune per model).

Suggested daily cap: **45940 tokens** (= action × 5). Treat as a single-developer upper bound for a dogfood session; reduce if running multiple plans in parallel.

