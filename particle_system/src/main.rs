//! particle_system — the app crate (42 subject v4.1). § 1: an empty window on the
//! engine, the particle count from the command line, honest fps in the title.

mod count;

use std::process::ExitCode;

use engine::{
    App, DispatchSize, Flow, FpsCounter, Frame, Gfx, GpuInfo, Input, Key, PresentPreference,
    RunError, WindowConfig, run, text_size,
};

use count::{Count, commas, parse_count, with_commas};

const DEFAULT_COUNT: u32 = 1_000_000;
/// Invocations per workgroup for the particle shaders (§ 2). A starting guess:
/// a multiple of every subgroup size, within what desktop GPUs allow. § 13 measures
/// the best value and § 16 checks it against the GPU's limits.
const LOCAL_SIZE: u32 = 256;
const BACKGROUND: [f32; 4] = [0.02, 0.02, 0.04, 1.0];
const ZERO_MESSAGE: &str = "Well..... This is what you asked for";

const USAGE: &str = "\
usage: particle_system [count] [--vsync]
  count    how many particles (default 1,000,000), e.g.
           1000000  1_000_000  500k  3m  1.5m  1e6  2^20  pi  42  lots  max
  --vsync  cap fps at the screen's refresh rate (default: uncapped)";

/// What was asked for on the command line.
struct Options {
    count: Count,
    present: PresentPreference,
}

/// Reads the arguments (without the program name). `Err` holds the message to print.
fn parse_args(args: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut count = None;
    let mut present = PresentPreference::Uncapped;
    for arg in args {
        match arg.as_str() {
            "--vsync" => present = PresentPreference::Vsync,
            "-h" | "--help" => return Err(String::new()),
            _ if count.is_some() => return Err(format!("unexpected argument '{arg}'")),
            _ => count = Some(parse_count(&arg)?),
        }
    }
    let count = count.unwrap_or(Count {
        value: DEFAULT_COUNT,
        note: None,
    });
    Ok(Options { count, present })
}

struct Particles {
    count: u32,
    fps: FpsCounter,
    /// The latest title, rebuilt when the fps counter gives a reading (twice a second).
    title: String,
}

impl Particles {
    fn new(count: u32) -> Self {
        Particles {
            count,
            fps: FpsCounter::new(0.5),
            title: format!("particle_system — {} particles", with_commas(count)),
        }
    }
}

impl App for Particles {
    fn setup(&mut self, gfx: &mut Gfx<'_>) -> Result<(), RunError> {
        // Nothing to create yet: the particle buffer arrives in § 4 — and never for a
        // count of 0. § 2: describe the GPU's compute side and the dispatch we'll need.
        for line in compute_report(&gfx.info(), self.count) {
            println!("{line}");
        }
        Ok(())
    }

    fn update(&mut self, input: &Input, dt: f32, _gfx: &mut Gfx<'_>) -> Flow {
        if input.pressed(Key::Escape) {
            return Flow::Exit;
        }
        if let Some(reading) = self.fps.tick(dt) {
            self.title = format!(
                "particle_system — {} particles — {:.0} fps ({:.2} ms)",
                with_commas(self.count),
                reading.fps,
                reading.frame_ms
            );
        }
        Flow::Continue
    }

    fn draw(&self, frame: &mut Frame) {
        frame.clear(BACKGROUND);
        frame.set_title(self.title.clone());
        if self.count == 0 {
            // Centred; 1–3 pixels per font texel depending on the window's width.
            let (width, height) = frame.size();
            let scale = (width / 900.0).clamp(1.0, 3.0).round();
            let (text_w, text_h) = text_size(ZERO_MESSAGE, scale);
            let position = ((width - text_w) / 2.0, (height - text_h) / 2.0);
            frame.draw_text(ZERO_MESSAGE, position, scale, [0.85, 0.85, 0.9]);
        }
    }
}

/// § 2: the GPU, its compute limits, and how `count` particles split into
/// workgroups of `LOCAL_SIZE` — plus a warning for anything that wouldn't fit.
fn compute_report(info: &GpuInfo, count: u32) -> Vec<String> {
    let (major, minor, patch) = info.vulkan_version;
    let [size_x, size_y, size_z] = info.max_workgroup_size;
    let mut lines = vec![
        format!(
            "GPU: {} (Vulkan {major}.{minor}.{patch}), queue family {} (graphics + present + compute), {}",
            info.name, info.queue_family, info.present_mode
        ),
        format!(
            "compute: workgroups of up to {} invocations ({size_x} × {size_y} × {size_z} per axis), \
             up to {} per axis per dispatch, {} KB shared memory, subgroup {}",
            info.max_workgroup_invocations,
            with_commas(info.max_workgroup_count[0]),
            info.max_shared_memory / 1024,
            info.subgroup_size
        ),
    ];

    if count == 0 {
        lines.push("dispatch: 0 particles — nothing to dispatch".to_string());
        return lines;
    }
    let d = DispatchSize::for_items(count, LOCAL_SIZE);
    lines.push(format!(
        "dispatch: {} particles at local size {LOCAL_SIZE} → {} workgroups, {} invocations, {} idle",
        with_commas(count),
        with_commas(d.workgroups),
        commas(d.invocations),
        with_commas(d.idle)
    ));

    if LOCAL_SIZE > info.max_workgroup_invocations || LOCAL_SIZE > size_x {
        lines.push(format!(
            "warning: local size {LOCAL_SIZE} is over this GPU's limit (§ 16 will pick one that fits)"
        ));
    }
    if info.subgroup_size > 0 && !LOCAL_SIZE.is_multiple_of(info.subgroup_size) {
        lines.push(format!(
            "warning: local size {LOCAL_SIZE} isn't a multiple of the subgroup size {} (wasted lanes)",
            info.subgroup_size
        ));
    }
    if d.workgroups > info.max_workgroup_count[0] {
        lines.push(format!(
            "warning: {} workgroups is over the {} allowed in one direction (needs a 2D dispatch)",
            with_commas(d.workgroups),
            with_commas(info.max_workgroup_count[0])
        ));
    }
    lines
}

fn main() -> ExitCode {
    let options = match parse_args(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            if !message.is_empty() {
                eprintln!("particle_system: {message}");
            }
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(note) = &options.count.note {
        println!("{note}");
    }

    let config = WindowConfig {
        title: "particle_system".to_string(),
        width: 1280,
        height: 800,
        present: options.present,
    };
    match run(config, Particles::new(options.count.value)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("particle_system: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> impl Iterator<Item = String> {
        list.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn defaults() {
        let options = parse_args(args(&[])).unwrap();
        assert_eq!(options.count.value, DEFAULT_COUNT);
        assert_eq!(options.present, PresentPreference::Uncapped);
    }

    #[test]
    fn count_and_vsync_in_any_order() {
        let options = parse_args(args(&["--vsync", "3m"])).unwrap();
        assert_eq!(options.count.value, 3_000_000);
        assert_eq!(options.present, PresentPreference::Vsync);
    }

    #[test]
    fn two_counts_or_junk_are_errors() {
        assert!(parse_args(args(&["1m", "2m"])).is_err());
        assert!(parse_args(args(&["lots of them"])).is_err());
    }
}
