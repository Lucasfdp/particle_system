*This project has been created as part of the 42 curriculum by luferna3.*

# particle_system

## Description

**particle_system** simulates and draws millions of particles on the GPU. Particles start
in a sphere or a cube and are pulled towards a gravity centre that sits under the cursor
or follows it; their colour depends on how far they are from the cursor.

Written in **Rust** on **Vulkan** (through MoltenVK on macOS). The particles live only in
video memory: they are created, moved and drawn by the GPU (compute and graphics shaders
sharing one buffer), never copied through RAM.

The code is split in two:

- **`engine/`** — a reusable rendering library, carried over from the scop project and
  extended here (compute pipelines, GPU synchronisation, performance tooling).
- **`particle_system/`** — the app: particle count, shapes, gravity, colours, controls.

## Instructions

> Work in progress — filled in at § 17 (mandatory finish).

```sh
make                 # check dependencies and build
make run N=1000000   # run with one million particles
```

## Resources

> Filled in as the project goes. Learning notes: [`learn/00-PLAN.md`](learn/00-PLAN.md).

### AI usage

- Learning material in `learn/` (explanations, questions) was written with Claude (Anthropic).
- Checkpoint code is written by Claude **after** I answer each section's questions, then
  walked through with me until I can explain every line. Sections and the code they
  produced are listed in `learn/PROGRESS.md`.
- The `engine/` crate was carried over from my scop project, built the same way.

## Learning

This project is also a structured learning plan: see [`learn/00-PLAN.md`](learn/00-PLAN.md)
(English) and [`learn_es/00-PLAN.md`](learn_es/00-PLAN.md) (Spanish).
