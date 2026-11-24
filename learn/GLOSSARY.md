# Glossary — particle_system

Terms in the order they were introduced. Plain language first, then precise.
Identifiers keep their English names. Terms already learned in scop are in scop's
`learn/GLOSSARY.md` and are not repeated here.

---

## Section 1 — A new app on the old engine

### release build
**Plain:** the fast version of the program, built for running, not debugging.
**Precise:** `cargo build --release`: `opt-level = 3`, no `debug_assertions`; in this
engine the validation layer is off. The only build performance is measured in.
**Introduced in:** § 1

### debug build
**Plain:** the slow, checked version of the program, built for finding bugs.
**Precise:** `cargo build`: minimal optimisation, overflow checks, `cfg!(debug_assertions)`
code included; the engine turns the Vulkan validation layer on.
**Introduced in:** § 1

### LTO
**Plain:** letting the compiler optimise across crate boundaries at the very end.
**Precise:** *link-time optimisation*. `lto = "thin"` in `[profile.release]`: a parallel,
cheaper variant of full ("fat") LTO; allows inlining `engine` functions into the app.
**Introduced in:** § 1

### codegen units
**Plain:** how many pieces a crate is cut into so it compiles in parallel.
**Precise:** *code generation units*. Fewer = slower builds, better optimisation;
`codegen-units = 1` gives the compiler the whole crate at once.
**Introduced in:** § 1

### frame time
**Plain:** how long one frame takes.
**Precise:** milliseconds between frames, `1000 / fps`. Compare optimisations in ms;
the subject's targets are 16.7 ms (60 fps) and 50 ms (20 fps).
**Introduced in:** § 1

### present mode
**Plain:** the rule for when a finished picture replaces the one on screen.
**Precise:** `VkPresentModeKHR`, chosen at swapchain creation from the list the surface
reports. Only `FIFO` is guaranteed.
**Introduced in:** § 1

### vsync
**Plain:** waiting for the screen's refresh before showing a new picture.
**Precise:** *vertical synchronisation* — syncing to the vertical blank between screen
refreshes. Caps fps at the refresh rate; prevents tearing.
**Introduced in:** § 1

### tearing
**Plain:** two frames shown at once, split by a horizontal line.
**Precise:** a new image presented while the display is part-way through scanning out
the previous one. Happens with `IMMEDIATE`.
**Introduced in:** § 1

### FIFO
**Plain:** "wait your turn": one new picture per screen refresh.
**Precise:** `VK_PRESENT_MODE_FIFO_KHR` — *first in, first out* queue of presented
images, one taken per vertical blank. Always supported; vsync on.
**Introduced in:** § 1

### IMMEDIATE
**Plain:** show the new picture right away, even mid-refresh.
**Precise:** `VK_PRESENT_MODE_IMMEDIATE_KHR` — no waiting, may tear. Optional.
**Introduced in:** § 1

### MAILBOX
**Plain:** keep only the newest picture waiting; show it at the next refresh.
**Precise:** `VK_PRESENT_MODE_MAILBOX_KHR` — like FIFO with a one-slot queue where a newer
image replaces the waiting one. Uncapped rendering without tearing. Optional.
**Introduced in:** § 1

### command-line argument
**Plain:** a word typed after the program's name when you start it.
**Precise:** `std::env::args()` — an iterator of `String`; index 0 is the program name.
**Introduced in:** § 1

---

## Section 2 — How a GPU runs a million things at once

### invocation
**Plain:** one GPU thread — one run of the shader.
**Precise:** a single execution of a compute shader's `main`, identified by its
`gl_GlobalInvocationID`.
**Introduced in:** § 2

### workgroup
**Plain:** a team of invocations you size yourself; they share fast memory.
**Precise:** a group of `local_size_x × y × z` invocations scheduled on one compute unit,
able to share `shared` memory and synchronise with `barrier()`.
**Introduced in:** § 2

### local size
**Plain:** how many invocations in one workgroup.
**Precise:** `layout(local_size_x = N) in;` in the shader. Limited by
`maxComputeWorkGroupInvocations` (≥ 128 guaranteed).
**Introduced in:** § 2

### dispatch
**Plain:** launching a compute job — the compute version of a draw call.
**Precise:** `cmd_dispatch(x, y, z)`: runs `x·y·z` workgroups of the bound compute pipeline.
**Introduced in:** § 2

### workgroup count
**Plain:** how many teams you launch.
**Precise:** the arguments of `cmd_dispatch`; `ceil(N / local_size)` for N items in 1D.
Limited per axis by `maxComputeWorkGroupCount` (≥ 65,535).
**Introduced in:** § 2

### global invocation ID
**Plain:** each invocation's own number across the whole dispatch.
**Precise:** `gl_GlobalInvocationID = gl_WorkGroupID × local size + gl_LocalInvocationID`.
**Introduced in:** § 2

### subgroup
**Plain:** the squad of invocations the hardware runs in lockstep (usually 32).
**Precise:** a hardware-defined subset of a workgroup executing as one SIMD unit. Vendor
names: warp (NVIDIA), wavefront (AMD), SIMD-group (Apple). Size from
`VkPhysicalDeviceSubgroupProperties`.
**Introduced in:** § 2

### SIMD
**Plain:** one instruction applied to many values at once.
**Precise:** *single instruction, multiple data*. A subgroup is a SIMD unit with one lane
per invocation.
**Introduced in:** § 2

### lockstep
**Plain:** everyone doing the same step at the same time.
**Precise:** all active lanes of a subgroup execute the same instruction together.
**Introduced in:** § 2

### divergence
**Plain:** invocations in one squad disagreeing on a branch — the squad does both.
**Precise:** lanes of one subgroup taking different control-flow paths; the subgroup
executes each path with the other lanes masked off.
**Introduced in:** § 2

### latency hiding
**Plain:** keeping busy with other squads while one waits for memory.
**Precise:** the compute unit switches to a ready subgroup whenever one stalls on a memory
access; needs many resident subgroups.
**Introduced in:** § 2

### occupancy
**Plain:** how many squads a counter can keep ready at once.
**Precise:** resident subgroups per compute unit ÷ the hardware maximum. Limited by
registers and shared memory per workgroup.
**Introduced in:** § 2

### bounds guard
**Plain:** the line that sends extra invocations home.
**Precise:** `if (id >= count) return;` at the top of a compute shader whose invocation
count is rounded up to whole workgroups.
**Introduced in:** § 2

### embarrassingly parallel
**Plain:** a job that splits into independent identical pieces with no coordination.
**Precise:** standard term for workloads with no dependencies between elements, like
updating each particle from its own data.
**Introduced in:** § 2

### compute unit
**Plain:** one "counter" of the GPU that runs several workgroups at once.
**Precise:** AMD's term; NVIDIA: streaming multiprocessor (SM); Apple: GPU core. Holds the
registers and shared memory its resident workgroups use.
**Introduced in:** § 2

### device limits
**Plain:** the numbers a GPU reports about what it can do.
**Precise:** `VkPhysicalDeviceLimits` (from `get_physical_device_properties`), plus
extension structs such as `VkPhysicalDeviceSubgroupProperties` via `…properties2`.
Vulkan guarantees minimums; real devices report more.
**Introduced in:** § 2

---

## Section 3 — Programs that compute instead of draw

### compute shader
**Plain:** a GPU program that computes instead of drawing: no vertices in, no pixels out.
**Precise:** a shader of the compute stage (`.comp`), run by `cmd_dispatch`; declares its
workgroup size with `layout(local_size_x…) in;` and works only through descriptors and
push constants.
**Introduced in:** § 3

### compute pipeline
**Plain:** the GPU object that holds one compute shader, ready to run.
**Precise:** created with `vkCreateComputePipelines` from a `VkComputePipelineCreateInfo`:
one shader stage (with optional specialisation info) and a pipeline layout.
**Introduced in:** § 3

### storage buffer
**Plain:** a buffer shaders can read and write, as big as you like.
**Precise:** descriptor type `STORAGE_BUFFER` (buffer usage `STORAGE_BUFFER`); declared in
GLSL with `buffer`, may end in an unsized array. Contrast: uniform buffers are small and
read-only.
**Introduced in:** § 3

### SSBO
**Plain:** another name for a storage buffer.
**Precise:** *shader storage buffer object* — the OpenGL term, common in tutorials.
**Introduced in:** § 3

### bind point
**Plain:** which kind of pipeline a bind command is for — drawing or computing.
**Precise:** `vk::PipelineBindPoint::GRAPHICS` / `::COMPUTE`. A command buffer keeps the
bound pipeline and descriptor sets separately per bind point.
**Introduced in:** § 3

### specialisation constant
**Plain:** a shader constant filled in by Rust when the pipeline is created.
**Precise:** `layout(constant_id = N) const …` (or `local_size_x_id = N` for the workgroup
size), set through `VkSpecializationInfo` (map entries + data bytes) on the shader stage.
Lets one SPIR-V file serve several configurations.
**Introduced in:** § 3

### readonly / writeonly
**Plain:** promises in the shader that a buffer is only read, or only written.
**Precise:** GLSL memory qualifiers on a `buffer` block; let the compiler optimise and
make intent explicit.
**Introduced in:** § 3

### test buffer
**Plain:** a throwaway buffer used only to check a shader's output.
**Precise:** a host-visible (and host-coherent) storage buffer the CPU maps after the
dispatch. Never used for particle data.
**Introduced in:** § 3

### readback
**Plain:** copying GPU results back to the CPU to look at them.
**Precise:** GPU writes → barrier (shader write → host read) → fence wait → map and read.
Slow; for tests and debugging only.
**Introduced in:** § 3
