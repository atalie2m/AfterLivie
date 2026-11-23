# 10. Risks and Gates

## 1. Risk Register

### R1. Shared Renderer Complexity

**Risk:** Building a shared renderer in v1.0 is technically heavy.

**Mitigation:** Validate in Phase 0, use proven libraries, scope renderer to overlays, build CLI harness, add golden tests early.

**Gate:** Continue only if CJK/emoji/multiline rendering and preview/export consistency are viable.

### R2. Renderer Performance

**Risk:** Dense comments or long videos make overlay rendering too slow.

**Mitigation:** Cache shaped runs, paragraph measurements, static row fragments, badges, and emotes. Render only visible comments. Profile dense fixtures early.

**Gate:** Dense 30-second fixture must render acceptably before v1.0 hardening.

### R3. Font and Emoji Fidelity

**Risk:** Font fallback, emoji assets, and missing glyphs vary by platform or license constraints.

**Mitigation:** Define default font policy, record font versions, diagnose missing glyphs, use bundled open-source fonts where feasible.

**Gate:** Font and asset licensing review before public release.

### R4. Media Backend Feasibility

**Risk:** Combining video decode, overlay composition, audio, export, progress, and cancellation becomes unstable.

**Mitigation:** Build full segment export in Phase 0. Keep media backend replaceable. Separate overlay rendering from media execution.

**Gate:** 10–60 second export must succeed with diagnostics before broader app work.

### R5. Source Video Quality Degradation

**Risk:** Scaling, color conversion, or encoding decisions visibly degrade source video.

**Mitigation:** Preserve native source region where possible, avoid fractional scaling, add frame comparison tests, diagnose color/HDR/VFR.

**Gate:** Default layout must pass visual and automated placement checks.

### R6. FFmpeg Packaging and Licensing

**Risk:** FFmpeg distribution affects licensing, notarization, app size, and updates.

**Mitigation:** Decide bundled vs external early. Isolate behind `ReplayMedia`. Document version. Review licenses.

**Gate:** Distribution model decision before v1.0 beta.

### R7. Swift/Rust Interop Complexity

**Risk:** Bridge introduces memory, build, or debugging problems.

**Mitigation:** Keep API coarse. Use generated bindings or disciplined C ABI. Test ownership and errors.

**Gate:** Bridge smoke and large payload tests must pass before UI expansion.

### R8. Scope Creep

**Risk:** Multi-source, styling, plugins, and Windows work delay v1.0.

**Mitigation:** Enforce scope partition. Keep multi-source in model, not UI. Move broad features to v1.x.

**Gate:** v1.0 backlog cannot include plugin runner or full multi-source UX.

### R9. Project File Instability

**Risk:** Early schema changes break projects.

**Mitigation:** Version schemas from the start, use fixtures, add migration tests.

**Gate:** Project load/save fixtures before v1.0 alpha.

### R10. Diagnostics Weakness

**Risk:** Users cannot understand failed imports, unsupported media, renderer issues, or export failures.

**Mitigation:** Structured diagnostics with severity, category, source reference, and actionable hint.

**Gate:** v1.0 cannot ship with opaque fatal errors for common failure classes.

## 2. Phase Gates

### Gate 0: Architecture Feasibility

Required:

- Swift calls Rust.
- Canonical import works.
- Renderer produces overlay frame.
- Media segment export succeeds.
- Diagnostics exist.

Decision:

- Go,
- Revisit renderer/media stack,
- Stop.

### Gate 1: Renderer Commitment

Required:

- CJK fixture passes.
- Emoji fixture passes.
- Golden images stable enough.
- Dense sample performance acceptable.
- Packaging/licensing plausible.

Decision:

- Adopt selected renderer stack,
- change stack,
- reduce renderer feature set,
- revisit architecture.

### Gate 2: v1.0 Alpha

Required:

- project package,
- default layout,
- preview export,
- final export,
- import/media/render diagnostics,
- save/open.

### Gate 3: v1.0 Beta

Required:

- cancellation,
- crash-safe saves,
- long sample testing,
- audio policy stable,
- docs and samples,
- no critical consistency mismatch.

### Gate 4: v1.0 Release

Required:

- release notes document limitations,
- renderer ADR finalized,
- no critical data loss,
- no critical export corruption,
- diagnostics acceptable,
- source quality strategy validated.

## 3. Decision Rules

- If renderer fails but media works, do not proceed to broad UI; revisit renderer stack.
- If media fails but renderer works, do not proceed to broad UI; revisit compositor/export path.
- If both work but performance is poor, profile before expanding scope.
- If project schema is unstable, do not add multi-source UI.
- If diagnostics are weak, do not add more import formats.
