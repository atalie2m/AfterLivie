# Native Livestream Archive Replay Composer — Documentation Set v2

**Date:** 2026-05-04  
**Status:** Revised planning set before implementation  
**Source:** Refactored from the original full proposal and the subsequent scope review.

## Purpose

This documentation set restructures the original proposal into execution-oriented documents. The central change is the separation of:

- **v1.0 release blockers**: the smallest credible product that proves the architecture and delivers production-grade single-source replay composition.
- **v1.x product expansion**: multi-source merge, broader importers, richer templates, and deeper styling.
- **future tracks**: plugin execution, advanced sync, automation, and Windows productization.

The major architectural commitment remains unchanged:

> The product should use a shared cross-platform overlay renderer from the beginning, because comment overlay quality, CJK/emoji behavior, and preview/export consistency are core product quality requirements.

## Document Index

| File | Purpose |
|---|---|
| `00_decision_summary.md` | Final decision summary and revised operating principles. |
| `01_product_proposal_v2.md` | Revised product proposal with a tighter v1.0 definition. |
| `02_scope_partition_release_plan.md` | Explicit split between v1.0, v1.x, and future tracks. |
| `03_architecture_overview.md` | System architecture, module boundaries, and invariants. |
| `04_renderer_adr.md` | Draft architecture decision record for the shared overlay renderer. |
| `05_phase0_technical_validation_plan.md` | Phase 0 vertical slice, acceptance tests, and kill/revisit criteria. |
| `06_implementation_plan.md` | Sequenced implementation plan from Phase 0 through v1.x. |
| `07_data_model_storage_schema.md` | Normalized comment model, project package, SQLite, and render-plan schemas. |
| `08_media_pipeline_plan.md` | Media probing, compatibility path, render pipeline, and audio policy. |
| `09_testing_validation_matrix.md` | Test matrix for core, renderer, media, UI, integration, and release gates. |
| `10_risks_and_gates.md` | Risk register, mitigations, and phase gates. |
| `11_backlog_v1x_future.md` | Post-v1.0 roadmap and intentionally deferred work. |
| `native_livestream_docs_v2_combined.md` | Combined single-file version of the full set. |

## Recommended Reading Order

1. `00_decision_summary.md`
2. `01_product_proposal_v2.md`
3. `02_scope_partition_release_plan.md`
4. `05_phase0_technical_validation_plan.md`
5. `04_renderer_adr.md`
6. `06_implementation_plan.md`

## Working Assumption

This set assumes implementation has not started. Therefore it treats architectural reset, scope separation, and idealized rebuilding as available options.
