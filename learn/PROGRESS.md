# Progress

**Current section:** 3
**Last session:** 2026-11-24

## Sections

| # | Status | Gate | Date | ES synced | Notes |
|---|---|---|---|---|---|
| 1 | done | passed after misses (Q-1.2: 4 ms = 250 fps, *higher* — ms and fps move opposite ways; Q-1.3: 250 fps = 4 ms not 0.4, gave ms not fps — fresh variant (5 ms → 200 fps, +5 ms → 100 fps) passed; Q-1.5: 0 isn't just 'no particles' — a zero-sized Vulkan buffer is invalid, so check before the GPU) — checkpoint code by Claude (engine: PresentPreference Vsync/Uncapped in WindowConfig, choose_present_mode picks IMMEDIATE → MAILBOX → FIFO from what's offered, kept for resize; app: count.rs parser, title with count/fps/ms, Esc) — 125 tests, clippy/fmt clean; walkthrough given after owner asked for the information first (do this every time: walk through, *then* ask); explain-back passed with misses (stored `present` = the app's preference kept for resize rebuilds, not device specs; fresh variant: default on rebuild → silently capped to 60 after a resize — owner thought validation would catch it: it can't, FIFO is valid, it's a logic bug); verified on Mac (0 → message window, lots → random count, --vsync → 60, uncapped resize: brief dip then back up) | 2026-11-24 | no | Owner's scope: fun parser (pi, 42, lots, max, googol, 1e6, 2^20, 1.5m); `0` opens an empty window saying "Well..... This is what you asked for" — no particle buffer is ever created for 0 |
| 2 | done | passed after misses (Q-2.1: subgroup size is the hardware's, no partial workgroups — count rounds up; Q-2.2: didn't compute, thought extras aren't dispatched and the guard makes groups whole — re-taught with 10 items / size 4, fresh variant (2M / 64 = 31,250, 0 idle) passed; Q-2.3: gave 'no wasted lanes' — it's divergence inside a subgroup; fresh variants: `< 32` (said diverges — no, line between squads), `< 16` (said both — only A), `< 48` (only B) passed; Q-2.4 precision: result 1…1M, fix = atomics; Q-2.5 precision: waiting on memory) — checkpoint code by Claude (engine: GpuInfo compute limits + subgroup size via properties2, queue family must also COMPUTE, compute.rs DispatchSize::for_items; app: startup compute report + warnings) — 129 tests, clippy/fmt clean; walkthrough given first; explain-back passed with misses (Q1: one *queue family* for all three → one queue, barriers only, no release/acquire between families — said 'device'; Q2: question was misleading (96 warns on nothing on the Mac; on 64-wide AMD it does) — fresh variant 16 @ 3M: got subgroup warning, missed 187,500 > 65,535 workgroups — 'always do the division'; follow-up (smallest multiple of 32 for 3M: 3M / 65,535 = 45.8 → 64) passed; Q3 precision: invocations = groups × size can pass u32::MAX); verified on Mac (Apple M2, Vulkan 1.2.357, family 0 does all three, 1024 invocations, 1,073,741,824 workgroups per axis — far above the 65,535 guarantee, 32 KB shared, subgroup 32; 3,907 groups / 192 idle). Nitpick fixed after: invocations printed with commas | 2026-11-24 | no | |
| 3 | written | — | 2026-11-24 | no | |
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
| Divergence = a branch line falling *inside* a subgroup (lanes of one squad disagree); different squads on different paths is free — fresh variant | § 2 | 0 | after § 4 |
| Extras: all invocations of the last group run; guard sends ids ≥ count home before touching the buffer — fresh variant | § 2 | 0 | after § 4 |
| Workgroup-count limit: always divide count ÷ local size and compare with 65,535 (e.g. 3M at 16 → 187,500, too many) — fresh variant | § 2 explain-back | 0 | after § 4 |
| fps ↔ ms: 1000 ÷ fps; same added ms costs far more fps at high fps — fresh numeric variant | § 1 | 0 | after § 3 |
| Validation catches *invalid* Vulkan use, not wrong-but-valid choices (FIFO after resize = silent logic bug) — fresh variant | § 1 explain-back | 0 | after § 3 |
| Zero-sized Vulkan buffer is invalid → handle 0 in the app, never create the buffer — fresh variant | § 1 | 0 | after § 3 |

## Open threads
- Engine copied from scop on 2026-11-24 (scop's engine at § 25). Not yet a git repo — owner to `git init` and commit.
- **Engine coupling found:** engine unit tests read `../resources/*.obj` (scop's files). Copied `resources/42.obj`, `teapot.obj` and `resources/test/` here so `cargo test` passes (112 tests, clippy clean, 2026-11-24). Fix properly in § 21: engine test fixtures inside `engine/`.
- Creativity extras: choose after § 17.
- GitHub issues configured (`learn/issues.json`, 21 sections + 10 tasks, 6 milestones). Claude's session had read-only GitHub access → owner runs the first `sync`.
