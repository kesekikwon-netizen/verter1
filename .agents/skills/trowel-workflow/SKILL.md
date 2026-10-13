---
name: trowel-workflow
description: Autonomous senior workflow for Trowel (verter1) vector illustration and archaeological CAD development. Integrates superpowers skills (brainstorming, writing-plans, executing-plans, TDD, systematic-debugging, code-review, verification) with strict clean-room, no-panic, and cargo xtask ci quality gates.
---

# Trowel Autonomous Development Workflow

## Core Principles
1. **Clean-Room**: Never use Adobe software or assets. Clean-room verification runs via `cargo xtask cleanroom`.
2. **Zero Panics**: No unwrap(), expect(), panic!, unreachable! in production crates. Return Result/Option.
3. **Everything is a Command**: Every user-visible operation is an engine Command with tests.
4. **Autonomous Self-Correction**: Test and verify autonomously before reporting. Never burden the user.
5. **Design 2.1 Consistency**: Strict visual and structural fidelity to the 1600 × 988 Trowel design.

## Development Lifecycle per Task
1. **Orient & Brainstorming**: Review design specs in `factory/jobs/` and activate `superpowers:brainstorming` when clarifying behavior.
2. **Specification & Plan**: Write exact testable criteria before coding (`superpowers:writing-plans`).
3. **Test-Driven Development**: Implement unit/integration tests before writing implementation code (`superpowers:test-driven-development`).
4. **Verification Gate**:
   - `cargo test -p <touched_crate>`
   - `cargo test -p vectorcraft-ui-egui trowel::`
   - `cargo xtask ci` (Linux/macOS fmt, clippy, test, assets, brands, layers, wasm)
5. **Record & Commit**: Update `factory/jobs/J###-<slug>.md` and `factory/backlog.md`, then commit and push to `origin main`.
