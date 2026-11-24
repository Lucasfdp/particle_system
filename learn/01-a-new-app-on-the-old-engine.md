---
section: 1
slug: a-new-app-on-the-old-engine
tier: T1
status: done
prereqs: []
terms: [release build, debug build, LTO, codegen units, frame time, present mode, vsync, tearing, FIFO, IMMEDIATE, MAILBOX, command-line argument]
---

# Section 1 — A new app on the old engine

**Depth: T1** — almost nothing here is new; the point is to start the project with
**honest numbers**. Every later performance section compares measurements, and a
measurement taken the wrong way is worse than none.
**You'll be able to:** start `particle_system` on the scop engine, choose how many
particles from the command line, and read an FPS figure that tells you what the GPU can
really do — not what the screen or the debug tools allow.

---

## The idea

You already have an engine that opens a window, runs a frame loop and shows the FPS in
the title. The new app plugs into it exactly as scop did: it implements `App`, and the
engine calls it every frame. For now it draws nothing — an empty window. That's enough
to set up the three things every later section relies on.

**1. How many particles.** The subject says the count must be *easily modifiable* for
testing. The obvious way: type it when you start the program —
`particle_system 3000000` — instead of editing the code and rebuilding.

**2. An FPS figure you can trust.** Picture a race where the runner is told "never go
faster than the pace car". Their time tells you about the pace car, not about them.
Your scop window ran at exactly 60 fps whatever you drew — the screen refreshes 60 times
a second, and the engine waits for each refresh before showing the next picture. That
waiting is the pace car. To find out how fast the GPU really is, the pace car has to go.

**3. A build that runs at full speed.** While developing, Rust builds a version full of
safety checks with almost no optimisation, and the engine switches on Vulkan's
validation layer — a checker that inspects every Vulkan call. Both are exactly what you
want when hunting bugs, and both can make the program several times slower. A number
measured in that build describes the checkers, not your code.

One more habit: compare **milliseconds per frame**, not frames per second. If one
change saves 2 ms, it saves 2 ms whether you were at 50 fps or at 300 fps — but in fps
the same saving looks like +5 in one case and +250 in the other.

## The names for it

What you just read is called a **release build** versus a **debug build**.

- **debug build** — `cargo build`. Little optimisation, overflow checks and
  `debug_assertions` on (*debug assertions*: the code inside `cfg!(debug_assertions)`
  only exists in this build). The engine turns the validation layer on only here (scop § 3).
- **release build** — `cargo build --release`. Full optimisation (`opt-level = 3`),
  validation layer off. **All performance numbers in this project come from release builds.**
- **LTO** — *link-time optimisation*. Normally each crate is optimised on its own; with
  LTO the compiler optimises across crates at the final link, so code in `engine` can be
  inlined into `particle_system`. `lto = "thin"` is a cheaper, parallel version of the full one.
- **codegen units** — *code generation units*: how many pieces the compiler splits a
  crate into to compile them in parallel. More pieces = faster builds, fewer cross-piece
  optimisations. `codegen-units = 1` = slowest build, best code.

The pace car is called **vsync** — *vertical synchronisation*: waiting for the screen's
refresh (old CRT screens drew top to bottom, then the beam went back up — the *vertical
blank*) before swapping in a new picture. In Vulkan you choose this behaviour with the
swapchain's **present mode** (*present* = hand a finished image to the screen):

| Mode | Name decoded | Behaviour | FPS limited by the screen? | Tearing? |
|---|---|---|---|---|
| `FIFO` | *first in, first out* — a queue of finished images, shown one per refresh | Waits for each refresh. Always supported. | yes | no |
| `IMMEDIATE` | shown *immediately*, mid-refresh if need be | Never waits. | no | yes |
| `MAILBOX` | like a *mailbox*: a newer letter replaces the one waiting | Never waits; the screen shows the newest finished image at each refresh, older ones are thrown away | no | no |

- **tearing** — the screen is halfway through showing one picture when the next one
  arrives, so the top half shows one frame and the bottom half the next, with a visible
  horizontal "tear" when things move.
- **frame time** — milliseconds from one frame to the next. `1000 / fps`. 60 fps = 16.7 ms.
- **command-line argument** — a word typed after the program name. Rust gives them to
  you with `std::env::args()`: element 0 is the program's own name, element 1 the first
  argument.

## How it actually works

**The new crate.** The workspace (`Cargo.toml` at the root) lists two members:
`engine` (copied from scop) and `particle_system` (new, empty `main`). The release
profile already sets `lto = "thin"` and `codegen-units = 1`.

**Present mode in the engine today.** `engine/src/swapchain.rs` has
`choose_present_mode`, which ignores what the GPU offers and always returns `FIFO`.
That was right for scop (smooth, no tearing, always supported). Here the engine needs a
**setting**: the app says what it prefers, the engine picks the best mode the device
actually supports.

```
preference = Uncapped:   IMMEDIATE if offered → else MAILBOX if offered → else FIFO
preference = Vsync:      FIFO (always offered)
```

`FIFO` is the only mode Vulkan guarantees; the others must be checked in the list the
surface reports (`get_physical_device_surface_present_modes`, scop § 4). MoltenVK
typically offers `FIFO` and `IMMEDIATE`.

**Where things belong (scop § 8's rule).** The *choice* "uncapped or vsync" is a
property of the window and swapchain → engine setting (`WindowConfig`). The *particle
count* means something only to this app → app code. The engine never learns what a
particle is until § 21 — and even then, only because several projects will want particles.

**Reading the count.** `std::env::args().nth(1)` gives `Option<String>`. Parse with
`str::parse::<u32>()`, which returns `Result`. Accept underscores (`1_000_000`) and the
suffixes `k`/`m` (`500k`, `3m`) so testing is quick. Reject 0 and anything unparsable
**before** the engine starts, with a usage message on stderr and a non-zero exit — no
GPU resource should ever be created with a nonsense size.

**The title.** The engine's `FpsCounter` (scop § 19) already averages over a short
window and `Frame::set_title` updates the title at most a few times a second (changing
the title is a call into the window system, not free). Add the frame time in ms and
the particle count: `particle_system — 1,000,000 particles — 412 fps (2.43 ms)`.

## Known traps

- **"It runs at exactly 60 fps whatever I do."** Vsync is on (FIFO). Not a performance
  result — a screen refresh rate. On a 120 Hz screen it'd be exactly 120.
- **Numbers measured with `cargo run`** (no `--release`): validation layer on,
  unoptimised Rust. Can be 5–20× slower than the real thing.
- **Comparing fps instead of ms.** "+100 fps" means nothing without the starting point.
- **`IMMEDIATE` asked for but not offered** → swapchain creation fails with a validation
  error. Always pick from the list the surface reports, with FIFO as the fallback.
- **Uncapped and the laptop fan.** With no cap the GPU runs flat out even when nothing
  changes. Fine for measuring; keep a vsync option for normal use.

## Checkpoint

1. `particle_system` implements `App`: empty window (clear colour), Esc quits.
2. Particle count from the first argument: default `1_000_000`; accepts `1500000`,
   `1_500_000`, `1500k`, `3m` (whole numbers only); rejects `0`, `1.5m` and junk with a
   usage line and exit code 1.
3. Engine: `WindowConfig` gains a present preference (`Vsync` / `Uncapped`);
   `choose_present_mode` picks from what the surface offers. The app uses `Uncapped`
   by default and `--vsync` switches back.
4. Title: count, fps and frame time in ms, e.g.
   `particle_system — 1,000,000 particles — 2850 fps (0.35 ms)`.

**Verify:** `cargo run --release -p particle_system -- 3m` shows far more than 60 fps
on your Mac; with `--vsync` it shows 60 (or your screen's rate);
`cargo run --release -p particle_system -- 0` prints usage and exits with code 1.

## Questions

Q-1.1 (recall) Name two things the debug build does that make its fps numbers useless
for judging this project's performance.

Q-1.2 (prediction) Your screen refreshes at 60 Hz and each frame takes the GPU 4 ms.
What does the title show with `FIFO`? With `IMMEDIATE`? What might you *see* on screen
with `IMMEDIATE` that you never see with `FIFO`?

Q-1.3 (prediction) Doubling the particles adds 4 ms to every frame. What fps do you get
after doubling if you started at 250 fps? And if you started at 50 fps? What does that
tell you about comparing optimisations in fps?

Q-1.4 (application) The particle count and the vsync/uncapped choice: which one is an
engine setting and which one is app code? Give the reason for each in one line.

Q-1.5 (prediction) Someone runs `particle_system lots`. What should happen, and why does
it matter that this is checked before the engine creates anything on the GPU?

Answers: `ANSWERS.md` § 1
