# Progress

**Current section:** 1
**Last session:** 2026-11-24

## Sections

| # | Status | Gate | Date | ES synced | Notes |
|---|---|---|---|---|---|
| 1 | written | — | 2026-11-24 | no | |
| 2 | written | — | 2026-11-24 | no | |
| 3 | not written | — | | no | |
| 4 | not written | — | | no | |
| 5 | not written | — | | no | |
| 6 | not written | — | | no | |
| 7 | not written | — | | no | |
| 8 | not written | — | | no | |
| 9 | not written | — | | no | |
| 10 | not written | — | | no | |
| 11 | not written | — | | no | |
| 12 | not written | — | | no | |
| 13 | not written | — | | no | |
| 14 | not written | — | | no | |
| 15 | not written | — | | no | |
| 16 | not written | — | | no | |
| 17 | not written | — | | no | |
| 18 | not written | — | | no | |
| 19 | not written | — | | no | |
| 20 | not written | — | | no | |
| 21 | not written | — | | no | |

## Prerequisite audit (2026-11-24)
- Known from scop: Vulkan setup, swapchain, frame loop, fences/semaphores, image
  barriers, pipelines, buffers + staging, descriptors, storage buffer (instancing),
  FPS counter, orbit camera, engine/app split.
- Known from C: threads, data races, mutexes.
- Diagnostic (9 questions):
  - Q1 dispatch sizing — no idea → § 2 full.
  - Q2 shared counter — race spotted; fix given as mutex (wrong on GPU) → §§ 2, 19.
  - Q3 no barrier between compute and draw — right that the draw can start early;
    missed that a barrier also makes the writes *visible* (caches) → § 7 (T3).
  - Q4 two frames in flight, one buffer — right (mixed/flickering frames) → § 8.
  - Q5 std430 size of two vec3 — no idea → § 4.
  - Q6 device-local vs host-visible — framed as flushing; it's where the memory
    physically sits (VRAM vs system RAM across the bus); unified memory point right → §§ 4, 16.
  - Q7 gravity singularity — no idea → § 9.
  - Q8 60 fps cap — right (FIFO, screen refresh) → § 1 at T0/T1.
  - Q9 cursor → 3D — depth needed (right), missed undoing the camera → § 10.
- Maths depth: T1 by owner's standing choice.

## Overrides
- 2026-11-24: method override carried over from scop — Claude writes checkpoint code
  after each gate, owner explains it back (DECISIONS.md).
- 2026-11-24: Spanish mirror paused (owner's choice, as in scop); `learn_es/` has the
  required files only, sections caught up in batches.
- 2026-11-24: optimisation added as an explicit goal (owner) → Phase 4 (§§ 12–16).

## Review queue

| Question | Source | Clean recalls | Next due |
|---|---|---|---|

## Open threads
- Engine copied from scop on 2026-11-24 (scop's engine at § 25). Not yet a git repo — owner to `git init` and commit.
- **Engine coupling found:** engine unit tests read `../resources/*.obj` (scop's files). Copied `resources/42.obj`, `teapot.obj` and `resources/test/` here so `cargo test` passes (112 tests, clippy clean, 2026-11-24). Fix properly in § 21: engine test fixtures inside `engine/`.
- Creativity extras: choose after § 17.
- GitHub issues configured (`learn/issues.json`, 21 sections + 10 tasks, 6 milestones). Claude's session had read-only GitHub access → owner runs the first `sync`.
