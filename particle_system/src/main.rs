//! particle_system — the app crate (42 subject v4.1). § 1: an empty window on the
//! engine, the particle count from the command line, honest fps in the title.

mod count;

use std::process::ExitCode;

use engine::{
    App, Flow, FpsCounter, Frame, Gfx, Input, Key, PresentPreference, RunError, WindowConfig, run,
    text_size,
};

use count::{Count, parse_count, with_commas};

const DEFAULT_COUNT: u32 = 1_000_000;
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
    fn setup(&mut self, _gfx: &mut Gfx<'_>) -> Result<(), RunError> {
        // Nothing yet: the particle buffer arrives in § 4 — and never for a count of 0.
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
