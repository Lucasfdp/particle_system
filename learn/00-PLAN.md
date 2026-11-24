# particle_system — Learning Plan

**Mode:** 42 (outer core, solo)
**Goal:** build a foundation in GPU computing and performance, and grow the engine
**Started:** 2026-11-24
**Stack:** Rust · Vulkan (`ash`, MoltenVK on macOS) · `winit` · GLSL → SPIR-V (`glslc`) · engine from scop
**Languages:** learn/ (EN) · learn_es/ (ES, paused — caught up in batches)

## What this project is for
Learning to use the GPU as a **general-purpose parallel computer**, not only a triangle
drawer: compute shaders, millions of threads, sharing one buffer between computing and
drawing safely, and **measuring and optimising** until it runs smoothly — on a strong
GPU and on a weak one. The engine gains a compute side, GPU timing tools and a
performance audit of what scop left behind.

## What this project is NOT for
- **Physics accuracy.** Gravity, integration and damping are taught at "works and looks
  right" (T1). Stable integrators, collisions and rigid bodies belong to **ft_newton**.
- **Rust CPU threads.** The parallel work here runs on the GPU. Multi-threaded CPU work
  (chunk meshing, streaming) belongs to **ft_vox**. Your C threading knowledge still
  transfers — to GPU races and atomics (§§ 2, 19).
- **Advanced GPU techniques at depth:** async compute queues, indirect draws beyond the
  emitter case, subgroup operations, GPU sorting. Named and explained once (T0) so you
  know when they'd matter.
- **Rendering quality.** Points with colour and blending, no lighting or materials.

## Constraints (subject v4.1)

**General**
- Recent GPU API, at least the latest stable major version (Vulkan ✓).
- **Particles are never allocated in RAM. Never, even at initialisation.** Everything
  lives in video memory.
- **1M particles at 60 fps, 3M at 20 fps** (minimum). Particle count easily changeable.
- **Interoperability, with the shared memory synchronised** ("acquire and release").
- Free language and window library. A Makefile or similar.

**Mandatory**
- Particles initialised as a **sphere or a cube**; switch shape with an input.
- Attracted to a **gravity centre**, turned on/off with an input; **static** (placed
  under the cursor when the key is pressed) or **following the cursor**.
- Particles in GPU memory (VRAM).
- **FPS printed on the window** (e.g. in the title).
- **Colours** depending at least on the distance between particles and the cursor.
- No mesh needed for particles.

**Bonus** (only graded if the mandatory part is perfect)
- Camera travelling through the particles, **WASD + mouse**.
- **Emitters** generating particles with a **life span**.
- Points for creativity. Performance limits don't apply — but don't fall to 10,000 at 20 fps.

**AI** — be transparent about AI use and identify what was AI-generated (README).

## Prerequisite audit (2026-11-24)
- Known from scop: Vulkan setup, swapchain, command buffers, basic sync (fences,
  semaphores, image barriers), pipelines, buffers + staging, descriptors, storage buffer
  (instancing), FPS counter, orbit camera.
- Known from C: threads, data races, mutexes.
- Diagnostic results: barrier purpose (✓, missing *visibility*), frames in flight race (✓),
  vsync cap (✓); GPU races (fix isn't a mutex — partial); device-local vs host-visible
  (partial — it's *where* the memory sits); cursor → 3D (partial — also undo the camera);
  dispatch sizing, std430 layout, gravity singularity: new.

## Depth allocation

Scores 1–3 on centrality, transferability, cost of learning later. Goal is "foundation
+ optimisation", so performance concepts with transferability 3 get +1. Owner's standing
override: maths at T1 ("a programmer, not a mathematician").

| Concept | Cent. | Trans. | Cost later | Total | Tier |
|---|---|---|---|---|---|
| Thinking in parallel (invocations, workgroups, subgroups) | 3 | 3 | 3 | 9 | T2 |
| Compute shaders + compute pipelines | 3 | 3 | 3 | 9 | T2 |
| Storage buffers + std430 layout | 3 | 3 | 2 | 8 | T2 |
| GPU-side initialisation, GPU random numbers | 3 | 2 | 1 | 6 | T1 |
| Compute → graphics sync ("interop") | 3 | 3 | 3 | 9 + defence | **T3** |
| Frames in flight with a shared buffer | 3 | 3 | 2 | 8 | T2 |
| Simulation maths (dt, integration, gravity) | 3 | 2 | 1 | 6 | T1 (maths override; ft_newton goes deeper) |
| Cursor → world point (unprojection) | 3 | 2 | 1 | 6 | T1 |
| Drawing points, blending, colour by distance | 3 | 2 | 1 | 6 | T1 |
| Measuring: GPU timestamps, frame capture | 3 | 3 | 3 | 9 +1 | T2 |
| Simulation speed: workgroup size, layout, bandwidth | 3 | 3 | 2 | 8 +1 | T2 |
| Drawing speed: fill rate, overdraw | 2 | 3 | 2 | 7 +1 | T2 |
| Engine overhead audit | 2 | 3 | 2 | 7 +1 | T2 |
| Adapting to the device (limits, weaker GPUs) | 2 | 3 | 2 | 7 +1 | T2 |
| Present modes (vsync) | 2 | 2 | 1 | 5 | T0 (known) |
| Engine compute abstraction + app settings | 3 | 3 | 2 | 8 | T2 |
| Bonus: free-fly camera | 2 | 2 | 1 | 5 | T1 (from the orbit camera) |
| Bonus: GPU atomics | 2 | 3 | 3 | 8 | T2 |
| Bonus: emitters, life span, GPU particle pool | 2 | 3 | 2 | 7 | T2 |
| Async compute, indirect draw, subgroup ops, GPU sort | 1 | 3 | 1 | 5 | T0 |

**Vehicle note:** particle_system is the right vehicle for **GPU compute, GPU
synchronisation and performance work** — one buffer, millions of elements, a hard
fps target, every change measurable. It is a poor vehicle for physics depth
(ft_newton) and for CPU multithreading (ft_vox).

## Sections

| # | File | Title | Tier | Status | Checkpoint |
|---|---|---|---|---|---|
| | **Phase 1 · Foundations** | | | | |
| 1 | `01-a-new-app-on-the-old-engine.md` | A new app on the old engine | T1 | done | `particle_system` app opens a window; uncapped FPS in the title; particle count from the command line |
| 2 | `02-thinking-in-parallel.md` | How a GPU runs a million things at once | T2 | done | Startup prints the device's compute limits and the dispatch size for N particles |
| 3 | `03-compute-shaders.md` | Programs that compute instead of draw | T2 | written | Engine compute pipeline; a compute shader fills a test buffer |
| 4 | `04-particle-buffer-layout.md` | The particle buffer and its memory layout | T2 | not written | Device-local particle buffer sized from N; Rust ↔ GLSL layout checked |
| | **Phase 2 · Particles on screen** | | | | |
| 5 | `05-creating-particles-on-the-gpu.md` | Creating particles without touching RAM | T1 | not written | Init compute shader: sphere / cube, switched by a key |
| 6 | `06-drawing-points.md` | Drawing a million points | T1 | not written | Vertex shader reads the particle buffer; sphere of points on screen |
| 7 | `07-compute-graphics-handover.md` | Handing the buffer from compute to graphics (the interop rule) | T3 | not written | Per-frame compute → draw with correct barriers; sync validation clean; defence note |
| 8 | `08-frames-in-flight-shared-buffer.md` | One buffer, two frames in flight | T2 | not written | Chosen strategy (single vs ping-pong) implemented and justified |
| | **Phase 3 · Simulation (mandatory)** | | | | |
| 9 | `09-moving-particles.md` | Making particles move: time, velocity, gravity | T1 | not written | Gravity centre on/off; particles orbit it without exploding |
| 10 | `10-cursor-to-world.md` | From the mouse pointer to a point in 3D | T1 | not written | Static and follow-the-cursor gravity modes |
| 11 | `11-colour-and-blending.md` | Colour by distance, and glowing points | T1 | not written | Colours depend on distance to the cursor; blending toggle |
| | **Phase 4 · Performance** | | | | |
| 12 | `12-measuring.md` | Measuring before optimising | T2 | not written | GPU timestamps per pass on screen; first frame capture |
| 13 | `13-fast-simulation.md` | Making the simulation fast | T2 | not written | Measured gains (workgroup size, layout, maths); 3M at ≥ 20 fps |
| 14 | `14-fast-drawing.md` | Making the drawing cheap | T2 | not written | Measured gains on fill rate / overdraw; 1M at ≥ 60 fps |
| 15 | `15-engine-overhead-audit.md` | Auditing the engine's per-frame work | T2 | not written | Per-frame CPU work in the inherited engine measured and trimmed |
| 16 | `16-adapting-to-the-device.md` | Running well on weaker machines | T2 | not written | Settings chosen from the device's limits; tested on campus machines |
| 17 | `17-mandatory-finish.md` | Making the mandatory part perfect | T1 | not written | `make` one-liner, README, defence checklist, self-evaluation |
| | **Phase 5 · Bonus** | | | | |
| 18 | `18-free-fly-camera.md` | Flying through the particles | T1 | not written | WASD + mouse camera |
| 19 | `19-gpu-atomics.md` | Many threads, one counter | T2 | not written | Atomic counter test; race shown and fixed |
| 20 | `20-emitters-and-life-span.md` | Emitters: particles that are born and die | T2 | not written | GPU particle pool, emitters, life span, no CPU readback |
| 21 | `21-engine-consolidation.md` | Moving the reusable parts into the engine | T1 | not written | Engine gains compute + particles + timing; app keeps only the subject's choices |

Sections 3–21 are written when you reach them, so the plan adapts to what turns out
to be hard. Creativity extras get their own sections once chosen (after § 17).

## Defence prep
<!-- Populated as sections complete. Source: DECISIONS.md + section traps. -->
- Interop in Vulkan: what replaces OpenCL's acquire/release (§ 7, DECISIONS.md).
- "Never in RAM": show that no particle data ever exists on the CPU side (§§ 4–5).

## Deferred / v2
- Separate compute queue (async compute) — only if § 12's measurements show the GPU idling.
- Skip staging for engine uploads when memory is `DEVICE_LOCAL | HOST_VISIBLE` (from scop's list; § 15).
