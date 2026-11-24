---
section: 3
slug: compute-shaders
tier: T2
status: not-started
prereqs: [2]
terms: [compute shader, compute pipeline, storage buffer, SSBO, bind point, specialisation constant, readonly / writeonly, test buffer, readback]
---

# Section 3 — Programs that compute instead of draw

**Depth: T2** — every particle in this project is created and moved by a compute shader.
You need to be able to build one from nothing, explain each piece, and recognise the
handful of mistakes that break it.
**You'll be able to:** write a compute shader, build the pipeline that runs it, record a
dispatch, and prove it worked — and say exactly how a compute pipeline differs from the
graphics pipelines you built in scop.

You already know most of the parts from scop: shaders compiled to SPIR-V (§ 6),
descriptor sets pointing at buffers (§ 10), push constants (§ 10/§ 15), command buffers
and submitting work (§ 5), the `run_once` upload helper (§ 7). This section is mostly
*fewer* pieces, not new ones.

---

## The idea

**A graphics pipeline is a factory line; a compute pipeline is one workbench.**
In scop, drawing went through a fixed line of stations: vertices come in, the vertex
shader places them, the rasteriser cuts triangles into pixels, the fragment shader
colours each pixel, and the result lands in an image on screen. You configured the line
(vertex layout, topology, blending, depth test, which image to draw into), but its
shape was fixed.

A compute pipeline throws all of that away and keeps one station: **your shader**. No
vertices arrive, no pixels come out, nothing is drawn. You say "run this program on
`N × 256` invocations", and each invocation gets only one thing for free: its own
number (§ 2). Everything else it must go and fetch itself, and anything it produces it
must write somewhere itself.

So a compute shader talks to the world in exactly two ways:
- **Small settings** handed in with the command — like a sticky note: "there are
  1,000,000 items". In Vulkan: push constants (you used them in scop for the model matrix).
- **Big memory** it can read and write — like a shelf of numbered boxes every invocation
  can reach: "box 517 is yours". In Vulkan: a buffer the shader is allowed to *write*,
  pointed at by a descriptor (scop § 10 used descriptors for read-only camera data and
  the instance matrices).

**Proving it worked.** A compute shader produces no picture, so how do you know it ran?
Before trusting it with particles, give it a throwaway job whose answer you know
("write `id × 2` into box `id`"), then copy a few boxes back to the CPU and check them.
This test buffer is *not* particle data — the "never in RAM" rule is about particles,
and the test exists to be read.

## The names for it

- **compute shader** — a shader of the *compute* stage, file extension `.comp`. Starts
  with `layout(local_size_x = …) in;` (§ 2) and has `main()` like any GLSL shader.
- **compute pipeline** — the Vulkan object that holds one compiled compute shader and
  its layout. Created with `vkCreateComputePipelines` from a
  `VkComputePipelineCreateInfo` (*Vk* = Vulkan; *create info* = the struct describing
  what to create), which has just **one** shader stage and a pipeline layout — compare
  scop § 6's graphics create-info with its dozen sub-structs.
- **storage buffer** — a buffer shaders can read *and write*, of any size, with an
  open-ended array at its end. Vulkan: descriptor type `STORAGE_BUFFER`, buffer usage
  `STORAGE_BUFFER`. In GLSL it's declared with the keyword `buffer` (a *uniform* buffer,
  scop § 10, is declared `uniform`: small, read-only, the same for every invocation).
- **SSBO** — *shader storage buffer object*: the OpenGL-era name for a storage buffer,
  still used everywhere in tutorials. Same thing.
- **bind point** — which kind of pipeline a bind command targets:
  `vk::PipelineBindPoint::GRAPHICS` or `::COMPUTE`. A command buffer remembers the bound
  pipeline and descriptor sets *separately for each*, so binding a compute pipeline
  doesn't unbind your graphics one.
- **specialisation constant** — a constant in the shader whose value is filled in by
  Rust *when the pipeline is created*, not when the shader is compiled. Written
  `layout(constant_id = 0) const uint X = 64;` (64 = default). The workgroup size has a
  shorthand: `layout(local_size_x_id = 0) in;`. *Specialise* = make a general shader
  specific to one use. (British spelling in prose; the Vulkan type is
  `VkSpecializationInfo`.)
- **readonly / writeonly** — GLSL memory qualifiers on a buffer: promises that the
  shader only reads (or only writes) it. The compiler can optimise with them, and
  validation can check you meant it.
- **test buffer** — a throwaway storage buffer, host-visible (scop § 7: memory the CPU
  can map), used only to check a shader's output.
- **readback** — copying GPU results back to the CPU to look at them. Slow, and the
  opposite of what this project does with particles. Fine for tests.

## How it actually works

### 1. The shader

```glsl
#version 450
layout(local_size_x_id = 0) in;              // workgroup size: set from Rust (spec constant 0)

layout(std430, set = 0, binding = 0) writeonly buffer Values {
    uint values[];                           // open-ended array: as long as the buffer
} data;

layout(push_constant) uniform Push {
    uint count;                              // how many values are real (§ 2)
} push;

void main() {
    uint id = gl_GlobalInvocationID.x;
    if (id >= push.count) return;            // bounds guard
    data.values[id] = id * 2u;               // each invocation writes only its own box
}
```

- `std430` is the rule for how the buffer's contents are laid out in memory (offsets,
  padding). For an array of `uint` it's simply 4 bytes each, no gaps. § 4 is all about it.
- `values[]` with no size: the array is as long as whatever buffer is bound. The shader
  can ask with `data.values.length()`, but passing `count` is clearer and the guard needs
  the *real* count, not the buffer's capacity.

### 2. Why the workgroup size comes from Rust

The Rust side computes the workgroup count from *its* idea of the local size
(`DispatchSize::for_items(count, LOCAL_SIZE)`, § 2). The shader runs with *its* idea.
If they disagree — Rust thinks 256, the shader says 128 — only half the invocations you
expected run, and nothing warns you. A specialisation constant makes Rust the single
source: the pipeline is created with `LOCAL_SIZE` filled in, so they can't disagree. It
also lets § 13 try different sizes and § 16 pick one per GPU without recompiling shaders.

```rust
let entry = vk::SpecializationMapEntry { constant_id: 0, offset: 0, size: 4 };
let data = LOCAL_SIZE.to_ne_bytes();         // ne = native endian: the CPU's own byte order
let spec = vk::SpecializationInfo::default()
    .map_entries(std::slice::from_ref(&entry))
    .data(&data);
```

### 3. The pipeline

Three objects, in order (you've built each kind before):

1. **Descriptor set layout** — "binding 0 is one storage buffer, used by the compute
   stage": `descriptor_type(STORAGE_BUFFER)`, `stage_flags(COMPUTE)`.
2. **Pipeline layout** — that set layout, plus a push-constant range of 4 bytes with
   `stage_flags(COMPUTE)`.
3. **Compute pipeline** — one stage: `stage(COMPUTE)`, the shader module, entry point
   `c"main"`, and the specialisation info.

Then, as in scop § 10: a descriptor pool, one descriptor set from it, and a descriptor
write pointing binding 0 at the test buffer.

### 4. Recording and running

```
bind pipeline         cmd_bind_pipeline(cmd, COMPUTE, pipeline)
bind descriptor set   cmd_bind_descriptor_sets(cmd, COMPUTE, layout, 0, &[set], &[])
push the count        cmd_push_constants(cmd, layout, COMPUTE, 0, &count.to_ne_bytes())
launch                cmd_dispatch(cmd, workgroups, 1, 1)
```

**Where it goes in a frame:** never between `cmd_begin_rendering` and
`cmd_end_rendering`. Those bracket drawing into images; a dispatch inside them is
invalid. Compute goes *before* the drawing starts (or after it ends). § 7 is about doing
that every frame safely; this section only needs it **once**.

For a one-off job the engine already has the right tool: `run_once` (scop § 7 — record a
command buffer, submit it, wait on a fence until the GPU is done). After the wait, the
CPU maps the test buffer and reads it.

### 5. Making the result visible to the CPU

"The GPU finished" (the fence) isn't quite "the CPU can see the numbers": the shader's
writes may still be sitting in GPU caches. One barrier at the end of the recorded
commands says *compute-shader writes must be made visible to host reads*:

```
src: COMPUTE_SHADER stage, SHADER_WRITE access
dst: HOST stage,           HOST_READ access
```

Plus host-visible memory that is also `HOST_COHERENT` (*coherent* = the CPU's view and
the GPU's view stay in agreement without manual flushes — scop § 7 picked this kind).
This is your first barrier between compute and something else; § 7 makes barriers the
main topic.

## Known traps

- **Dispatch inside rendering** → validation error ("must not be called inside a render
  pass instance"). Record compute before `cmd_begin_rendering`.
- **Wrong bind point** (`GRAPHICS` when binding the compute pipeline or its sets) → the
  dispatch runs with nothing bound: validation error at the dispatch.
- **Push-constant range without the `COMPUTE` stage flag** → error at pipeline creation
  (the shader uses a block the layout doesn't offer it). Same trap as scop § 15.
- **Descriptor type mismatch** (`UNIFORM_BUFFER` in the layout, `buffer` in the shader) →
  error at pipeline creation.
- **Local size hard-coded in the shader and in Rust** → they drift apart; part of the
  data silently never gets processed. Use the specialisation constant.
- **Reading the test buffer right after submit** (no fence wait) → you see zeros or old
  values: submit returns before the GPU even starts (scop § 7).
- **Forgetting `.comp` in `build.rs`** → the shader is never compiled; the
  `include_bytes!` of its `.spv` fails the build with "file not found".

## Checkpoint

Claude writes this after the gate (method override), then walks through it:

1. `engine/build.rs` (and the Makefile's `spv` target, already done) compile `.comp` too,
   with the committed fallback in `shaders/spv/` like the other shaders. *(Closes the
   "build.rs compiles `.comp`" blocker issue.)*
2. Engine `compute.rs`: a compute pipeline that takes SPIR-V, a local size
   (specialisation constant 0), one storage buffer at set 0 binding 0, and a small
   push-constant block. `Gfx` gains what an app needs to create one, plus a storage
   buffer, and to run a dispatch once and wait.
3. Engine: a test helper that creates a host-visible test buffer, runs `fill_test.comp`
   (the shader above) on it once, and returns the values.
4. App: `particle_system --compute-test [count]` runs it for `count` values (default
   1,000,000), checks every one is `id × 2`, prints `compute test: OK (1,000,000 values,
   3,907 workgroups)` or the first wrong index, and exits. Normal runs don't touch it.

**Verify:** `cargo run --release -p particle_system -- --compute-test` prints OK;
`-- --compute-test 1000` and `-- --compute-test 1048576` too (non-multiple and multiple
of 256); a debug run (`cargo run -p particle_system -- --compute-test`) is silent from the
validation layer.

## Questions

Q-3.1 (recall) Name three things a graphics pipeline has that a compute pipeline doesn't.
And what are the only two ways a compute shader receives its inputs?

Q-3.2 (prediction) You record `cmd_dispatch` between `cmd_begin_rendering` and
`cmd_end_rendering`. What happens, and where should it go instead?

Q-3.3 (prediction) Rust sizes the dispatch with `LOCAL_SIZE = 256`, but the shader
hard-codes `layout(local_size_x = 128) in;`. You run it for 1,000,000 values. Roughly how
many values get written, which ones, and does anything warn you? What stops this from
happening in the checkpoint?

Q-3.4 (application) Your test reads back zeros everywhere, even though the dispatch ran.
Give two different causes from this section, and how you'd tell them apart.

Q-3.5 (prediction) The pipeline layout's push-constant range is declared with
`stage_flags(VERTEX)` (copied from scop). The shader is a compute shader using
`push.count`. When does it fail — at shader compile time, pipeline creation, or the
dispatch — and why then?

Answers: `ANSWERS.md` § 3
