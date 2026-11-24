# Answers — particle_system

Read after attempting. Answers explain the reasoning, not just the fact.

## Section 1

**A-1.1** Any two of: (a) the validation layer is on — it inspects every Vulkan call,
which costs CPU time per call; (b) Rust code is barely optimised (`opt-level = 0`): no
inlining, every iterator and bounds check kept; (c) overflow checks and
`debug_assertions` code run. All three slow the program for reasons that have nothing
to do with the GPU work you're trying to measure.

**A-1.2** `FIFO`: **60 fps** — the GPU finishes in 4 ms and then waits for the next
refresh; the title measures the screen. `IMMEDIATE`: about **250 fps** (1000 / 4) — the
GPU's real speed. With `IMMEDIATE` you may see **tearing**: a horizontal line where the
top part of the screen shows one frame and the bottom part the next, most visible when
things move quickly sideways.

**A-1.3** 250 fps = 4 ms per frame → 8 ms → **125 fps** (−125). 50 fps = 20 ms →
24 ms → **≈ 42 fps** (−8). Same cost, wildly different-looking drops. fps is the inverse
of time, so equal savings or costs look huge at high fps and tiny at low fps. Compare
milliseconds; convert to fps only to check the subject's targets (60 fps = 16.7 ms,
20 fps = 50 ms).

**A-1.4** **Vsync/uncapped → engine setting**: it's about the swapchain, which belongs
to the engine, and every app will want the same choice (ft_vox, ft_newton).
**Particle count → app**: only particle_system knows what a particle is; the engine
just creates buffers of whatever size the app asks for.

**A-1.5** Print a usage line (`usage: particle_system [count]  e.g. 1000000, 500k, 3m`)
to stderr and exit with a non-zero code, without opening a window. Checking first means
the error is clear and immediate, and no GPU resource is ever created with a nonsense
size: `0` would make zero-sized buffers (invalid in Vulkan — a validation error, or
undefined behaviour without validation), and an unchecked parse failure would panic
mid-setup with a less helpful message, possibly after creating half the Vulkan objects.

## Section 2

**A-2.1** **invocation** (one thread — the hardware runs one per lane) < **subgroup**
(fixed by the hardware: 32 on your Mac; you don't choose it) < **workgroup** (you choose
it in the shader: `local_size_x`) < **dispatch** (you choose the workgroup count when you
record `cmd_dispatch`). The dispatch is the whole job.

**A-2.2** `ceil(3,000,000 / 128)` = **23,438 workgroups** (23,437.5 rounded up).
Total invocations: 23,438 × 128 = **3,000,064**, so **64** are extra. They must hit the
bounds guard — `if (id >= count) return;` — as the first thing they do, before reading or
writing the particle buffer, otherwise they read and write past its end.

**A-2.3** **Shader B is faster.** In A, every subgroup of 32 contains 16 even and 16 odd
invocations, so every subgroup runs *both* paths one after the other (lanes switched off
in turn): the expensive path runs for every subgroup, with half the lanes idle.
In B, all 32 invocations of a subgroup belong to the same workgroup, so they all take the
same branch: no divergence, half the subgroups run only the expensive path, half only the
cheap one. On the expensive path, A spends roughly **twice** the time B does (every
subgroup pays it, instead of half of them).

**A-2.4** Anything from **1 to 1,000,000** (in practice, a small number, far below a
million). Each invocation reads the value, adds one in a register, writes it back; many
read the same old value at the same moment and their writes overwrite each other —
the classic lost update you know from C threads. A mutex doesn't work because waiting
means blocking: a blocked invocation stalls its whole subgroup in lockstep, and the
invocation holding the lock may be in a subgroup that can't run until the waiting one
finishes → deadlock (the GPU hangs or the driver kills it as *device lost*). The GPU
answer is an **atomic** operation (`atomicAdd`) — one indivisible read-add-write done by
the memory system — or, better, a design where invocations don't share the value at all
(§ 19).

**A-2.5** **Slower.** With one invocation per lane, there are no spare subgroups: when a
subgroup waits hundreds of cycles for memory, nothing else is ready, and the lanes sit
idle. With one invocation per particle, each compute unit has many subgroups queued and
switches to a ready one whenever another waits — memory waits are hidden. (A modest
loop — a few particles per invocation — can sometimes help by reusing setup work, but
only while enough subgroups remain to hide latency. § 13 measures this.)
