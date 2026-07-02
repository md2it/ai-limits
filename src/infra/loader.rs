use std::env;
use std::io::{self, IsTerminal, Write};
use std::time::Duration;

use chrono::{DateTime, Local};

const TOP_FRAME: &str = "=-=-=-=-=-=-=-=-=-=-=-=-= AI LIMITS =-=-=-=-=-=-=-=-=-=-=-=-";
const FRAME_WIDTH: usize = 60;
const BOTTOM_DECORATION: &str = "=-=-=-=-=-=-=-=-=";
const LOADER_SHOW_DELAY: Duration = Duration::from_millis(350);
const UNICODE_SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const ASCII_SPINNER_FRAMES: [&str; 4] = ["-", "\\", "|", "/"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalStatus {
    Done,
    Part,
    Fail,
}

impl TerminalStatus {
    fn label(self) -> &'static str {
        match self {
            TerminalStatus::Done => "DONE",
            TerminalStatus::Part => "PART",
            TerminalStatus::Fail => "FAIL",
        }
    }
}

pub struct TerminalUi {
    interactive: bool,
    unicode: bool,
    loader_lines: usize,
    static_loaders_rendered: bool,
}

impl TerminalUi {
    pub fn new() -> Self {
        let interactive = io::stdout().is_terminal();
        let unicode = interactive && environment_is_utf8();

        Self {
            interactive,
            unicode,
            loader_lines: 0,
            static_loaders_rendered: false,
        }
    }

    pub fn print_top(&mut self) -> io::Result<()> {
        println!();
        println!("{TOP_FRAME}");
        println!();
        Ok(())
    }

    pub fn print_bottom(&mut self, status: TerminalStatus) -> io::Result<()> {
        let frame = format_bottom_frame(status, Local::now());

        println!();
        println!("{frame}");
        println!();
        Ok(())
    }

    pub fn print_provider_heading(&mut self, heading: &str) -> io::Result<()> {
        println!();
        println!("            ---------- {heading} ----------");
        println!();
        Ok(())
    }

    pub fn print_provider_block(&mut self, heading: &str, body: &str) -> io::Result<()> {
        self.clear_loaders()?;
        self.print_provider_heading(heading)?;
        print!("{}", body.trim_end());
        println!();
        println!();
        io::stdout().flush()
    }

    pub fn render_loaders(&mut self, loaders: &[LoaderView<'_>]) -> io::Result<()> {
        if self.interactive {
            self.clear_loaders()?;
            for loader in loaders {
                println!("{}", self.format_loader(loader));
            }
            self.loader_lines = loaders.len();
            io::stdout().flush()?;
            return Ok(());
        }

        if self.static_loaders_rendered {
            return Ok(());
        }

        for loader in loaders {
            println!("{}", self.format_loader(loader));
        }
        self.static_loaders_rendered = true;
        io::stdout().flush()
    }

    pub fn finish_loaders(&mut self) -> io::Result<()> {
        self.clear_loaders()
    }

    fn clear_loaders(&mut self) -> io::Result<()> {
        if !self.interactive || self.loader_lines == 0 {
            self.loader_lines = 0;
            return Ok(());
        }

        move_cursor_up(self.loader_lines);
        for index in 0..self.loader_lines {
            print!("\x1b[2K\r");
            if index + 1 < self.loader_lines {
                print!("\x1b[1B");
            }
        }
        move_cursor_up(self.loader_lines.saturating_sub(1));
        self.loader_lines = 0;
        io::stdout().flush()
    }

    fn format_loader(&self, loader: &LoaderView<'_>) -> String {
        format!("{} waiting {}", self.spinner(loader.frame), loader.label)
    }

    fn spinner(&self, frame: usize) -> String {
        if self.unicode {
            UNICODE_SPINNER_FRAMES[frame % UNICODE_SPINNER_FRAMES.len()].to_string()
        } else {
            ASCII_SPINNER_FRAMES[frame % ASCII_SPINNER_FRAMES.len()].to_string()
        }
    }
}

impl Drop for TerminalUi {
    fn drop(&mut self) {
        let _ = self.finish_loaders();
    }
}

pub struct LoaderView<'a> {
    pub label: &'a str,
    pub frame: usize,
}

pub fn loader_tick() -> Duration {
    Duration::from_millis(180)
}

pub fn loader_show_delay() -> Duration {
    LOADER_SHOW_DELAY
}

fn environment_is_utf8() -> bool {
    ["LC_ALL", "LC_CTYPE", "LANG"].iter().any(|key| {
        env::var(key)
            .map(|value| value.to_ascii_uppercase().contains("UTF-8"))
            .unwrap_or(false)
    })
}

fn move_cursor_up(lines: usize) {
    if lines > 0 {
        print!("\x1b[{lines}A");
    }
}

fn format_bottom_frame(status: TerminalStatus, completed_at: DateTime<Local>) -> String {
    let body = format!(
        " {} {} ",
        status.label(),
        completed_at.format("%Y-%m-%d %H:%M:%S")
    );
    let decoration_width = FRAME_WIDTH
        .checked_sub(body.len())
        .expect("bottom frame body must fit into frame width");
    let side_width = decoration_width / 2;

    debug_assert_eq!(decoration_width % 2, 0);
    debug_assert_eq!(BOTTOM_DECORATION.len(), side_width);

    format!("{BOTTOM_DECORATION}{body}{BOTTOM_DECORATION}")
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};

    use super::{format_bottom_frame, TerminalStatus, BOTTOM_DECORATION, FRAME_WIDTH};

    #[test]
    fn bottom_frame_keeps_width_and_symmetric_decoration() {
        let completed_at = Local
            .with_ymd_and_hms(2026, 7, 2, 15, 4, 5)
            .single()
            .expect("valid local time");

        for status in [
            TerminalStatus::Done,
            TerminalStatus::Part,
            TerminalStatus::Fail,
        ] {
            let frame = format_bottom_frame(status, completed_at);

            assert_eq!(frame.len(), FRAME_WIDTH);
            assert!(frame.starts_with(BOTTOM_DECORATION));
            assert!(frame.ends_with(BOTTOM_DECORATION));
        }

        assert_eq!(
            format_bottom_frame(TerminalStatus::Done, completed_at),
            "=-=-=-=-=-=-=-=-= DONE 2026-07-02 15:04:05 =-=-=-=-=-=-=-=-="
        );
    }
}
