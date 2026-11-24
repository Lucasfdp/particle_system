# Decisions

One entry per real decision. These double as defence prep: "why did you choose this
over the obvious alternative?" is the question evaluators actually ask.

Decisions inherited with the engine (Rust + Vulkan, `ash`, `winit`, dynamic rendering,
`build.rs` + `glslc`, the `App` trait, …) are recorded in scop's `DECISIONS.md`. Only
new ones go here.

## 2026-11-24 — Copy the engine into this repo (not a shared crate)
**Chose:** copy scop's `engine/` crate into `particle-system/engine/` and keep growing it here.
**Rejected:** a separate engine repo pulled in as a git/path dependency (one engine for
every project, but 42 evaluates only what is inside the repository, and a moving
external dependency could break a finished project); a crate inside the scop workspace
(ties an unrelated project to scop's repo).
**Why:** the repo stays self-contained for evaluation; the engine can change freely
without touching a project that is already submitted.
**Cost accepted:** scop's engine and this one drift apart. Improvements made here don't
reach scop (that's fine: scop is finished). The next project copies from here.
**Revisit if:** three or more projects need the same engine at once.

## 2026-11-24 — Vulkan compute for both simulation and rendering (no OpenCL/CUDA)
**Chose:** compute shaders (GLSL → SPIR-V) in the same Vulkan device that draws.
**Rejected:** OpenCL + OpenGL interop (the "classic" version of this subject — two APIs,
OpenCL deprecated on macOS); CUDA (NVIDIA only, no Mac); Metal (Mac only).
**Why:** the subject (v4.1) lists Vulkan explicitly; one API means the particle buffer
is a single allocation used by both the compute and the graphics pipeline, with no copy.
**Cost accepted:** the subject's "interoperability" wording ("memory acquire and release
functions") is written for OpenCL/OpenGL. The Vulkan equivalent — barriers and, with a
separate compute queue, queue-family ownership transfer (*release* on one queue,
*acquire* on the other) — must be shown and explained at the defence (§ 7).
**Revisit if:** an evaluator rejects single-API interop — then the honest fallback is
a separate compute queue family with explicit release/acquire barriers.

## 2026-11-24 — Learning-plan method and Spanish mirror carried over from scop
**Chose:** owner reads each section and passes its gate first; Claude then writes the
checkpoint code and walks through it; the section is done once the owner explains every
line back. `learn_es/` paused and caught up in batches.
**Why:** same reasons as scop (hand-writing every line was too slow; understanding is
what's tested). The subject asks that AI-generated work be identified → README AI usage.
**Cost accepted:** the defence depends on the explain-back being honest.

## 2026-11-24 — Optimisation as an explicit goal
**Chose:** a whole phase of the plan (§§ 12–16) on measuring and optimising — including
the engine code inherited from scop — with the aim of running cleanly on weaker GPUs.
**Why:** owner's goal; this is the first project where performance is graded and
measurable (millions of particles), so it is the right vehicle.
**Cost accepted:** more sections before the bonus.
