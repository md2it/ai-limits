use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CODEX_COMMAND: &str = "codex";
const EXPECT_COMMAND: &str = "expect";
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);
const PROCESS_TIMEOUT: Duration = Duration::from_secs(60);

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ai-usage: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let diagnostics = Diagnostics::create()?;
    let expect_script = expect_script();
    diagnostics.event("runtime_start")?;
    diagnostics.event(&format!("spawn command={EXPECT_COMMAND} args=-c <script>"))?;
    diagnostics.write_expect_script(&expect_script)?;
    diagnostics.write_stdin_sent(
        "bracketed-paste /status\\r\nwait\nbracketed-paste /status\\r\nctrl-c\n",
    )?;

    let mut child = Command::new(EXPECT_COMMAND)
        .args(["-c", &expect_script])
        .env("TERM", "xterm-256color")
        .env("COLUMNS", "120")
        .env("LINES", "40")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    diagnostics.event(&format!("process_started pid={}", child.id()))?;

    let stdout_reader = child
        .stdout
        .take()
        .map(|stream| read_stream(stream, diagnostics.clone(), "stdout"))
        .expect("stdout is piped");
    let stderr_reader = child
        .stderr
        .take()
        .map(|stream| read_stream(stream, diagnostics.clone(), "stderr"))
        .expect("stderr is piped");

    let started_at = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            diagnostics.event("process_finished")?;
            break;
        }

        if started_at.elapsed() >= PROCESS_TIMEOUT {
            diagnostics.event("process_timeout kill")?;
            child.kill()?;
            let _ = child.wait();
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    thread::sleep(SHUTDOWN_WAIT);

    let stdout = stdout_reader.join().unwrap_or_default();
    let stderr = stderr_reader.join().unwrap_or_default();

    diagnostics.write_cleaned(&stdout)?;
    let cleaned_stdout = clean_terminal_output(&stdout);
    let compacted_stdout = compact_terminal_text(&cleaned_stdout);
    let usage_summary = extract_usage_summary(&compacted_stdout);
    diagnostics.event(&format!(
        "runtime_finish stdout_bytes={} stderr_bytes={} diagnostics_dir={}",
        stdout.len(),
        stderr.len(),
        diagnostics.dir().display()
    ))?;

    if let Some(summary) = usage_summary {
        println!("{summary}");
    } else {
        println!("Codex usage: not found in CLI output");
    }

    if !stderr.trim().is_empty() {
        eprint!("{stderr}");
    }

    println!("ai-usage diagnostics: {}", diagnostics.dir().display());

    Ok(())
}

fn expect_script() -> String {
    format!(
        r#"set timeout 20
log_user 1
spawn env TERM=xterm-256color COLUMNS=120 LINES=40 sh -c {{stty cols 120 rows 40; exec {CODEX_COMMAND} --no-alt-screen}}
expect {{
    -re {{OpenAI Codex}} {{}}
    timeout {{}}
}}
after 2000
send "\033\[200~/status\033\[201~\r"
expect {{
    -re {{Credits:}} {{set have_usage 1}}
    -re {{refresh requested|5h limit:|Weekly limit:}} {{set have_usage 0}}
    timeout {{set have_usage 0}}
}}
if {{$have_usage == 0}} {{
    after 3000
    send "\033\[200~/status\033\[201~\r"
    expect {{
        -re {{Credits:}} {{}}
        timeout {{}}
    }}
}}
after 1000
send "\003"
expect {{
    eof {{}}
    timeout {{exit 0}}
}}
"#
    )
}

#[derive(Clone)]
struct Diagnostics {
    dir: PathBuf,
    events: Arc<Mutex<File>>,
}

impl Diagnostics {
    fn create() -> io::Result<Self> {
        let dir = runtime_dir()?;
        fs::create_dir_all(&dir)?;

        let events = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("events.log"))?;

        Ok(Self {
            dir,
            events: Arc::new(Mutex::new(events)),
        })
    }

    fn dir(&self) -> &Path {
        &self.dir
    }

    fn event(&self, message: &str) -> io::Result<()> {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let mut events = self.events.lock().expect("events log lock is poisoned");
        writeln!(events, "{elapsed} {message}")?;
        events.flush()
    }

    fn raw_path(&self, stream_name: &str) -> PathBuf {
        self.dir.join(format!("{stream_name}.raw"))
    }

    fn write_cleaned(&self, stdout: &str) -> io::Result<()> {
        let cleaned = clean_terminal_output(stdout);
        fs::write(self.dir.join("stdout.cleaned.txt"), cleaned.as_bytes())?;
        fs::write(
            self.dir.join("stdout.compacted.txt"),
            compact_terminal_text(&cleaned),
        )
    }

    fn write_stdin_sent(&self, text: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join("stdin.sent.log"))?;
        file.write_all(text.as_bytes())?;
        file.flush()
    }

    fn write_expect_script(&self, text: &str) -> io::Result<()> {
        fs::write(self.dir.join("expect.script.tcl"), text)
    }
}

fn runtime_dir() -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let process_id = std::process::id();
    Ok(std::env::current_dir()?
        .join(".runtime")
        .join("ai-usage")
        .join(format!("{timestamp}-{process_id}")))
}

fn read_stream<R>(
    mut stream: R,
    diagnostics: Diagnostics,
    stream_name: &'static str,
) -> thread::JoinHandle<String>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut raw_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(diagnostics.raw_path(stream_name))
            .ok();
        let mut output = String::new();
        let mut buffer = [0_u8; 4096];

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    let _ = diagnostics.event(&format!("{stream_name}_closed"));
                    break;
                }
                Ok(count) => {
                    let bytes = &buffer[..count];
                    if let Some(file) = raw_file.as_mut() {
                        let _ = file.write_all(bytes);
                        let _ = file.flush();
                    }

                    output.push_str(&String::from_utf8_lossy(bytes));
                    let _ = diagnostics.event(&format!("{stream_name}_chunk bytes={count}"));
                }
                Err(error) => {
                    let _ = diagnostics.event(&format!("{stream_name}_read_error {error}"));
                    break;
                }
            }
        }

        output
    })
}

fn clean_terminal_output(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut cleaned = String::new();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            0x1b => {
                index += 1;
                if index >= bytes.len() {
                    break;
                }

                match bytes[index] {
                    b'[' => {
                        index += 1;
                        while index < bytes.len() && !bytes[index].is_ascii_alphabetic() {
                            index += 1;
                        }
                        index += 1;
                    }
                    b']' => {
                        index += 1;
                        while index < bytes.len() {
                            if bytes[index] == 0x07 {
                                index += 1;
                                break;
                            }
                            if bytes[index] == b'\\'
                                && index > 0
                                && bytes[index.saturating_sub(1)] == 0x1b
                            {
                                index += 1;
                                break;
                            }
                            index += 1;
                        }
                    }
                    _ => {
                        index += 1;
                    }
                }
            }
            b'\r' | b'\n' | b'\t' => {
                cleaned.push(bytes[index] as char);
                index += 1;
            }
            byte if byte.is_ascii_control() => {
                index += 1;
            }
            _ => {
                let rest = &input[index..];
                if let Some(character) = rest.chars().next() {
                    cleaned.push(character);
                    index += character.len_utf8();
                } else {
                    break;
                }
            }
        }
    }

    cleaned
}

fn compact_terminal_text(input: &str) -> String {
    let mut output = String::new();
    let mut pending_word = String::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.chars().count() == 1 {
            pending_word.push_str(trimmed);
            continue;
        }

        flush_pending_word(&mut output, &mut pending_word);

        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(trimmed);
    }

    flush_pending_word(&mut output, &mut pending_word);

    if !output.is_empty() {
        output.push('\n');
    }

    output
}

fn flush_pending_word(output: &mut String, pending_word: &mut String) {
    if pending_word.is_empty() {
        return;
    }

    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(pending_word);
    pending_word.clear();
}

fn extract_usage_summary(input: &str) -> Option<String> {
    let mut five_hour_limit = None;
    let mut weekly_limit = None;
    let mut credits = None;

    for raw_line in input.lines() {
        let line = raw_line
            .trim()
            .trim_matches(|character| character == '\u{2502}')
            .trim();
        let normalized = line.split_whitespace().collect::<Vec<_>>().join(" ");

        if normalized.starts_with("5h limit:") {
            five_hour_limit = Some(normalized);
        } else if normalized.starts_with("Weekly limit:") {
            weekly_limit = Some(normalized);
        } else if normalized.starts_with("Credits:") {
            credits = Some(normalized);
        }
    }

    if five_hour_limit.is_none() && weekly_limit.is_none() && credits.is_none() {
        return None;
    }

    let mut summary = String::from("Codex usage:\n");

    if let Some(value) = five_hour_limit {
        summary.push_str(&value);
        summary.push('\n');
    }
    if let Some(value) = weekly_limit {
        summary.push_str(&value);
        summary.push('\n');
    }
    if let Some(value) = credits {
        summary.push_str(&value);
        summary.push('\n');
    }

    Some(summary)
}
