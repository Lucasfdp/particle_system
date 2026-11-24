---
section: 2
slug: thinking-in-parallel
tier: T2
status: not-started
prereqs: [1]
terms: [invocation, workgroup, local size, dispatch, workgroup count, global invocation ID, subgroup, SIMD, lockstep, divergence, latency hiding, occupancy, bounds guard, embarrassingly parallel, compute unit, device limits]
---

# Section 2 — How a GPU runs a million things at once

**Depth: T2** — every later section (compute shaders, sync, atomics, all of Phase 4)
assumes this picture. You need to be able to *predict* what the GPU does with your work,
not just follow a pattern.
**You'll be able to:** say how many threads a dispatch launches and how they're grouped,
spot the code shapes that make a GPU slow (branches that split a group, too few threads
to hide memory waits), and explain why GPU threads don't use locks.

You know threads from C. Keep that picture — most of it transfers — and watch for the
three places where the GPU is different: **how many**, **how they're grouped**, and
**how they wait**.

---

## The idea

**One chef or a thousand cooks.** A CPU is a handful of brilliant chefs (8–16 cores):
each can cook anything, take decisions, jump between recipes. A GPU is a kitchen of
thousands of line cooks who are individually slow and simple, but who all do *the same
step at the same time* on different plates. If your job is "slice 3 million carrots",
the thousand cooks win by a mile. If your job is "invent a menu", the chefs win.

Particles are the carrot job: every particle runs exactly the same little program —
read my position and velocity, pull towards the gravity point, move, write back — and no
particle needs to know what another one is doing. Jobs like this, which split into
independent identical pieces with no coordination, are the GPU's favourite food.

**Cooks work in squads, squads in teams.** The cooks aren't independent. They stand in
**squads of 32** (on most GPUs) and a squad shares one instruction-reader: the squad
does step 1 together, then step 2 together, like rowers following one drum. If the
recipe says "if the carrot is bent, peel it first", the squad can't split: the cooks
with straight carrots *stand idle* while the others peel, then everyone continues.
Branches cost you the time of *both* paths whenever cooks in one squad disagree.

Squads are grouped into **teams** that you size yourself (say 256 cooks = 8 squads).
A team works at one counter and shares a small, very fast tray (shared memory) that the
team can use to pass things between members. Different teams can't talk to each other
during the job.

**Waiting without stopping.** Fetching something from video memory takes the GPU
hundreds of clock cycles — an eternity. A CPU thread that waits on memory just stalls
(caches try to stop this happening). The GPU's trick is different: when a squad has to
wait for its ingredients, the counter instantly switches to *another squad* that's ready
to work, then another. As long as there are plenty of squads queued up, the counter
never idles. That's why a GPU wants **far more threads than it has cooks**: the extra
ones are what keep it busy while the others wait.

**Why no locks.** In C you'd protect a shared counter with a mutex: one thread holds
it, the others sleep until it's released. On a GPU, "sleep until released" would mean
a squad sitting at the counter, blocked, with its 31 squad-mates stuck in lockstep —
and possibly waiting for a cook in a squad that isn't even running yet. Deadlocks and
collapse. So GPU code is designed so that threads **don't share writable data at all**
(each particle writes only its own slot), and when they genuinely must (a shared
counter), the hardware offers single indivisible operations instead of locks — that's
§ 19.

## The names for it

| In the kitchen | Vulkan / GLSL name | Decoded |
|---|---|---|
| one cook doing one plate | **invocation** | one *invoking* (calling) of the shader — one GPU thread |
| a squad of 32 in lockstep | **subgroup** | a *sub*-division of the work*group* the hardware runs together. Vendors' names: *warp* (NVIDIA, from weaving: the threads stretched side by side), *wavefront* (AMD), *SIMD-group* (Apple) |
| a team you size | **workgroup** | a *group* of invocations doing the *work* together, sharing memory |
| team size | **local size** | `layout(local_size_x = 256) in;` — size of one workgroup, set in the shader |
| how many teams | **workgroup count** | given when you launch, e.g. `cmd_dispatch(count_x, 1, 1)` |
| launching the whole job | **dispatch** | to *dispatch* = send off. The compute equivalent of a draw call |
| a cook's plate number | **global invocation ID** | `gl_GlobalInvocationID` = workgroup number × local size + position inside the group. *gl_* = OpenGL heritage, kept in GLSL |
| one counter (hardware) | **compute unit** | AMD's name; NVIDIA: *SM* (streaming multiprocessor); Apple: *GPU core*. Runs several workgroups at once |

- **SIMD** — *single instruction, multiple data*: one instruction applied to many values
  at once. A subgroup is SIMD in hardware: one instruction, 32 lanes of data.
- **lockstep** — moving together, step for step (from soldiers marching with locked steps).
- **divergence** — when invocations in one subgroup take different branches. The
  subgroup runs both paths, switching off the lanes that didn't take each one.
- **latency hiding** — covering the wait (*latency*) for memory by switching to other
  subgroups that are ready.
- **occupancy** — how many subgroups a compute unit can keep queued at once, out of the
  maximum. Low occupancy = not enough spare squads to hide waits. It drops when each
  invocation needs lots of registers or shared memory, because the counter has a fixed
  amount to share out.
- **bounds guard** — the line at the top of the shader that sends extra invocations home
  (`if (id >= count) return;`).
- **embarrassingly parallel** — the standard (and real) term for problems that split
  into independent pieces with no coordination. "Embarrassingly" because there's no
  cleverness needed to parallelise them.
- **device limits** — the numbers each GPU reports about itself
  (`VkPhysicalDeviceLimits`: *Vk* = Vulkan, *physical device* = the actual GPU).

## How it actually works

**Counting a dispatch.** You choose the local size in the shader; at launch you choose
the workgroup count. The total number of invocations is always *count × local size* —
whole workgroups only. With N particles:

```
workgroup_count = ceil(N / local_size)       // in integers: (N + local_size - 1) / local_size
invocations     = workgroup_count * local_size   // ≥ N
```

Unless N is an exact multiple, the last workgroup has spare invocations whose ID is
≥ N. They still run. Without the bounds guard they'd read and write past the end of the
particle buffer — on a GPU that's not a tidy segfault, it's silent garbage, a device
lost error (*the GPU gave up*), or nothing at all, depending on the driver.

```glsl
layout(local_size_x = 256) in;
void main() {
    uint id = gl_GlobalInvocationID.x;
    if (id >= push.count) return;   // bounds guard
    // ... particle[id] only
}
```

**Three dimensions, one used.** Local size and workgroup count are 3D (`x, y, z`),
because images and volumes are naturally 2D/3D. A particle list is 1D: use `x` only,
leave `y` and `z` at 1.

**What the limits say** (minimums Vulkan *guarantees*; real GPUs usually report more):

| Limit | Guaranteed at least | Meaning |
|---|---|---|
| `maxComputeWorkGroupInvocations` | 128 | biggest local size (x·y·z) |
| `maxComputeWorkGroupSize` | 128 × 128 × 64 | biggest local size per axis |
| `maxComputeWorkGroupCount` | 65,535 per axis | most workgroups per axis in one dispatch |
| `maxComputeSharedMemorySize` | 16 KB | the team's shared tray |
| subgroup size | (no minimum: 1+) | 32 on NVIDIA and Apple, 32 or 64 on AMD, 8–32 on Intel |

Subgroup size isn't in `VkPhysicalDeviceLimits`; it comes from
`VkPhysicalDeviceSubgroupProperties` (Vulkan 1.1+), read by chaining it onto
`get_physical_device_properties2` (the *2* = the extensible version of the call that
accepts extra structs).

Two consequences for later:
- A local size of 256 works on every real desktop GPU, but **Vulkan only promises 128**.
  § 16 picks the size from the limits instead of assuming.
- 1D dispatch caps you at 65,535 workgroups. With local size 256 that's ~16.7M
  particles — far above the targets, but with local size 64 it's ~4.2M, uncomfortably
  close. Larger counts need the y dimension too.

**Making local size a multiple of the subgroup size.** A workgroup of 48 on a
32-wide GPU fills one subgroup and half of another; the empty half-squad still occupies
hardware. Local sizes are therefore multiples of 32 (64 to be safe on AMD), and 64–256
is the usual sweet spot — § 13 *measures* which.

**Divergence in practice.** It only costs when lanes *in the same subgroup* disagree.
A branch on `id % 2` splits every subgroup in half (both paths, every time); a branch
that's the same for a whole workgroup (`gl_WorkGroupID.x % 2`) costs nothing extra. Most
particle code has no real branches beyond the bounds guard — keep it that way.

**Where your C threads fit.** A pthread is expensive to create, scheduled by the OS,
can block, and you have a few dozen. An invocation is nearly free, scheduled by the
hardware, must not block, and you launch millions. The race conditions are the same
(two writers, one address — undefined result); the fixes are not (no mutexes: give each
invocation its own slot, or use atomics, § 19).

## Known traps

- **Missing bounds guard** → out-of-range writes. Symptom: works at N = 1,048,576
  (a multiple of 256), corrupts or crashes at N = 1,000,000.
- **Local size not a multiple of 32** → wasted lanes, slower for no reason.
- **Asking for a local size above the device's limit** → pipeline creation fails
  (validation error); on a driver without validation, undefined.
- **Dispatching N workgroups instead of N / local size** → 256× too many invocations;
  the guard saves correctness but you burn the GPU on empty work.
- **"More threads per workgroup is always faster."** Not if each needs many registers:
  occupancy drops and memory waits stop being hidden. Measure (§ 13).
- **Treating C intuition as GPU truth**: "I'll spin-wait for the other thread" can hang
  the GPU forever, because the thread you wait for may not be scheduled until you finish.

## Checkpoint

The engine learns to describe the GPU's compute side; no shaders yet.

1. Engine: `GpuInfo` (already printed at startup by scop) gains
   `max_workgroup_invocations`, `max_workgroup_size: [u32; 3]`,
   `max_workgroup_count: [u32; 3]`, `max_shared_memory`, `subgroup_size`.
   Subgroup size via `get_physical_device_properties2` + `PhysicalDeviceSubgroupProperties`.
2. Engine: confirm the queue family chosen in scop § 3 also supports `COMPUTE`
   (Vulkan guarantees some family does both graphics and compute if graphics exists —
   check it's *this* one, and error clearly if not).
3. App: at startup print, for the chosen N and a local size of 256, the workgroup count,
   total invocations, and how many invocations the bounds guard will send home; warn if
   the workgroup count exceeds `max_workgroup_count[0]`.

**Verify:** `cargo run --release -p particle_system -- 1000000` prints your Mac's limits
(expect 1024 invocations, subgroup 32) and `3907 workgroups, 1000192 invocations, 192 idle`;
`-- 1048576` prints `0 idle`.

## Questions

Q-2.1 (recall) Put these in order of size, smallest first, and say who decides the size
of each: workgroup, invocation, dispatch, subgroup.

Q-2.2 (application) 3,000,000 particles, `local_size_x = 128`. How many workgroups do you
dispatch, how many invocations run in total, and what exactly must the extra ones do?

Q-2.3 (prediction) Shader A branches on `gl_GlobalInvocationID.x % 2` (even → an
expensive path, odd → a cheap one). Shader B does the same work but branches on
`gl_WorkGroupID.x % 2`. Both handle the same particles. Which is faster on a 32-wide GPU,
and roughly by how much on the expensive path?

Q-2.4 (prediction) A million invocations each run `data.counter = data.counter + 1;` on
the same value in a buffer, starting at 0. What values could you read afterwards? Why
isn't "wrap it in a mutex, like in C" the answer on a GPU?

Q-2.5 (prediction) Your Mac's GPU has a few thousand lanes. Someone rewrites the shader
so that exactly one invocation runs per lane, each looping over a few hundred particles —
"no wasted threads". Same total work. Faster or slower than one invocation per particle,
and why?

Answers: `ANSWERS.md` § 2
