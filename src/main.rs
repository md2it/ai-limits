use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CODEX_COMMAND: &str = "codex";
const CLAUDE_COMMAND: &str = "claude";
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
    diagnostics.event("runtime_start")?;

    let codex_result = run_provider(
        &diagnostics,
        "codex",
        None,
        &codex_expect_script(),
        "bracketed-paste /status\\r\nwait\nbracketed-paste /status\\r\nctrl-c\n",
    )?;
    let codex_summary = extract_codex_usage_summary(&codex_result.compacted_stdout);

    let claude_result = run_provider(
        &diagnostics,
        "claude",
        Some("claude"),
        &claude_expect_script(),
        "accept default theme if first-run wizard appears\n/usage\\r\nctrl-c twice\n",
    )?;
    let claude_summary = extract_claude_usage_summary(&claude_result.compacted_stdout);

    diagnostics.event(&format!(
        "runtime_finish diagnostics_dir={}",
        diagnostics.dir().display()
    ))?;

    if let Some(summary) = codex_summary {
        println!("{summary}");
    } else {
        println!("Codex usage: not found in CLI output");
    }

    if let Some(summary) = claude_summary {
        println!("{summary}");
    } else {
        println!("Claude usage: not found in CLI output");
    }

    if !codex_result.stderr.trim().is_empty() {
        eprint!("{}", codex_result.stderr);
    }
    if !claude_result.stderr.trim().is_empty() {
        eprint!("{}", claude_result.stderr);
    }

    println!("ai-usage diagnostics: {}", diagnostics.dir().display());

    Ok(())
}

struct ProviderRun {
    compacted_stdout: String,
    stderr: String,
}

fn run_provider(
    diagnostics: &Diagnostics,
    provider: &'static str,
    file_prefix: Option<&'static str>,
    expect_script: &str,
    stdin_sent: &str,
) -> io::Result<ProviderRun> {
    diagnostics.event(&format!(
        "{provider} spawn command={EXPECT_COMMAND} args=-c <script>"
    ))?;
    diagnostics.write_expect_script(file_prefix, expect_script)?;
    diagnostics.write_stdin_sent(file_prefix, stdin_sent)?;

    let mut child = Command::new(EXPECT_COMMAND)
        .args(["-c", expect_script])
        .env("TERM", "xterm-256color")
        .env("COLUMNS", "120")
        .env("LINES", "40")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    diagnostics.event(&format!("{provider} process_started pid={}", child.id()))?;

    let stdout_reader = child
        .stdout
        .take()
        .map(|stream| read_stream(stream, diagnostics.clone(), provider, file_prefix, "stdout"))
        .expect("stdout is piped");
    let stderr_reader = child
        .stderr
        .take()
        .map(|stream| read_stream(stream, diagnostics.clone(), provider, file_prefix, "stderr"))
        .expect("stderr is piped");

    let started_at = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            diagnostics.event(&format!("{provider} process_finished"))?;
            break;
        }

        if started_at.elapsed() >= PROCESS_TIMEOUT {
            diagnostics.event(&format!("{provider} process_timeout kill"))?;
            child.kill()?;
            let _ = child.wait();
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    thread::sleep(SHUTDOWN_WAIT);

    let stdout = stdout_reader.join().unwrap_or_default();
    let stderr = stderr_reader.join().unwrap_or_default();

    let cleaned_stdout = clean_terminal_output(&stdout);
    let compacted_stdout = compact_terminal_text(&cleaned_stdout);
    diagnostics.write_cleaned(file_prefix, &cleaned_stdout, &compacted_stdout)?;
    diagnostics.event(&format!(
        "{provider} runtime_finish stdout_bytes={} stderr_bytes={}",
        stdout.len(),
        stderr.len()
    ))?;

    Ok(ProviderRun {
        compacted_stdout,
        stderr,
    })
}

fn codex_expect_script() -> String {
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

fn claude_expect_script() -> String {
    format!(
        r#"set timeout 25
log_user 1
spawn env TERM=xterm-256color COLUMNS=120 LINES=40 sh -c {{stty cols 120 rows 40; exec {CLAUDE_COMMAND} --no-chrome}}
expect {{
    -re {{Choose.*text.*style|Syntax theme}} {{
        send "\r"
        exp_continue
    }}
    -re {{for shortcuts|Do you trust|Select login method}} {{}}
    timeout {{}}
}}
after 500
send -- "/usage\r"
expect {{
    -re {{Usage|Current session|Current week|Resets}} {{}}
    -re {{Select login method|Choose.*text.*style}} {{}}
    timeout {{}}
}}
after 10000
send "\003"
after 500
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

    fn file_name(prefix: Option<&str>, name: &str) -> String {
        match prefix {
            Some(prefix) => format!("{prefix}.{name}"),
            None => name.to_string(),
        }
    }

    fn raw_path(&self, prefix: Option<&str>, stream_name: &str) -> PathBuf {
        self.dir
            .join(Self::file_name(prefix, &format!("{stream_name}.raw")))
    }

    fn write_cleaned(
        &self,
        prefix: Option<&str>,
        cleaned: &str,
        compacted: &str,
    ) -> io::Result<()> {
        fs::write(
            self.dir.join(Self::file_name(prefix, "stdout.cleaned.txt")),
            cleaned.as_bytes(),
        )?;
        fs::write(
            self.dir
                .join(Self::file_name(prefix, "stdout.compacted.txt")),
            compacted,
        )
    }

    fn write_stdin_sent(&self, prefix: Option<&str>, text: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join(Self::file_name(prefix, "stdin.sent.log")))?;
        file.write_all(text.as_bytes())?;
        file.flush()
    }

    fn write_expect_script(&self, prefix: Option<&str>, text: &str) -> io::Result<()> {
        fs::write(
            self.dir.join(Self::file_name(prefix, "expect.script.tcl")),
            text,
        )
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
    provider: &'static str,
    prefix: Option<&'static str>,
    stream_name: &'static str,
) -> thread::JoinHandle<String>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut raw_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(diagnostics.raw_path(prefix, stream_name))
            .ok();
        let mut output = String::new();
        let mut buffer = [0_u8; 4096];

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    let _ = diagnostics.event(&format!("{provider} {stream_name}_closed"));
                    break;
                }
                Ok(count) => {
                    let bytes = &buffer[..count];
                    if let Some(file) = raw_file.as_mut() {
                        let _ = file.write_all(bytes);
                        let _ = file.flush();
                    }

                    output.push_str(&String::from_utf8_lossy(bytes));
                    let _ =
                        diagnostics.event(&format!("{provider} {stream_name}_chunk bytes={count}"));
                }
                Err(error) => {
                    let _ =
                        diagnostics.event(&format!("{provider} {stream_name}_read_error {error}"));
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

fn extract_codex_usage_summary(input: &str) -> Option<String> {
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

fn extract_claude_usage_summary(input: &str) -> Option<String> {
    let lines = input
        .split(['\n', '\r'])
        .map(normalize_terminal_line)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if lines.iter().any(|line| {
        let compact = compact_for_matching(line);
        compact.contains("selectloginmethod") || compact.contains("choosethetextstyle")
    }) {
        return Some(
            "Claude usage:\nClaude CLI is not ready: interactive setup is required\n".to_string(),
        );
    }

    let current_session = extract_claude_limit_block(&lines, "Current session");
    let current_week = extract_claude_limit_block(&lines, "Current week");
    let total_cost = find_line_by_compact_prefix(&lines, "totalcost");
    let token_usage = find_line_by_compact_prefix(&lines, "usage");

    if current_session.is_none()
        && current_week.is_none()
        && total_cost.is_none()
        && token_usage.is_none()
    {
        return None;
    }

    let mut summary = String::from("Claude usage:\n");

    if let Some(value) = current_session {
        summary.push_str(&value);
        summary.push('\n');
    }

    if let Some(value) = current_week {
        summary.push_str(&value);
        summary.push('\n');
    }

    if let Some(value) = total_cost.and_then(|line| format_claude_metric_line(&line)) {
        summary.push_str(&value);
        summary.push('\n');
    }

    if let Some(value) = token_usage.and_then(|line| format_claude_metric_line(&line)) {
        summary.push_str(&value);
        summary.push('\n');
    }

    Some(summary)
}

fn normalize_terminal_line(raw_line: &str) -> String {
    raw_line
        .trim()
        .trim_matches(|character| character == '\u{2502}')
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_claude_limit_block(lines: &[String], label: &str) -> Option<String> {
    let label_compact = compact_for_matching(label);
    let start = lines
        .iter()
        .position(|line| compact_for_matching(line).starts_with(&label_compact))?;
    let percent = lines
        .iter()
        .skip(start + 1)
        .take(3)
        .find_map(|line| extract_percent_used(line))?;
    let resets = lines
        .iter()
        .skip(start + 1)
        .take(5)
        .find_map(|line| extract_resets(line));

    let mut summary = format!("{label}: {percent} used");
    if let Some(resets) = resets {
        summary.push_str(" (resets ");
        summary.push_str(&format_claude_reset(&resets));
        summary.push(')');
    }

    Some(summary)
}

fn extract_percent_used(line: &str) -> Option<String> {
    let used_index = line.find("used")?;
    let before_used = &line[..used_index];
    let percent_index = before_used.rfind('%')?;
    let digits = before_used[..percent_index]
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();

    if digits.is_empty() {
        return None;
    }

    Some(digits.chars().rev().collect::<String>() + "%")
}

fn format_claude_metric_line(line: &str) -> Option<String> {
    let compact = compact_for_matching(line);

    if compact.starts_with("totalcost") {
        let value = line.split_once(':')?.1.trim();
        return Some(format!("Total cost: {value}"));
    }

    if compact.starts_with("usage") {
        let value = line.split_once(':')?.1.trim();
        return Some(format!("Tokens: {}", readable_compact_usage(value)));
    }

    None
}

fn readable_compact_usage(value: &str) -> String {
    let mut output = String::new();
    let mut previous = None;

    for character in value.chars() {
        if character == ',' {
            output.push_str(", ");
        } else {
            if character.is_ascii_alphabetic()
                && previous
                    .map(|previous: char| previous.is_ascii_digit())
                    .unwrap_or(false)
            {
                output.push(' ');
            }
            output.push(character);
        }
        previous = Some(character);
    }

    output
        .replace("cacheread", "cache read")
        .replace("cachewrite", "cache write")
}

fn format_claude_reset(value: &str) -> String {
    let with_timezone_space = value.replace('(', " (");
    let mut chars = with_timezone_space.chars().peekable();
    let mut month = String::new();

    while chars
        .peek()
        .map(|character| character.is_ascii_alphabetic())
        .unwrap_or(false)
    {
        month.push(chars.next().expect("peeked character exists"));
    }

    let mut day = String::new();
    while chars
        .peek()
        .map(|character| character.is_ascii_digit())
        .unwrap_or(false)
    {
        day.push(chars.next().expect("peeked character exists"));
    }

    let rest = chars.collect::<String>();
    if month.len() == 3 && !day.is_empty() && rest.starts_with("at") {
        return format!("{month} {day} at {}", rest.trim_start_matches("at"));
    }

    with_timezone_space
}

fn extract_resets(line: &str) -> Option<String> {
    let compact = compact_for_matching(line);
    if !compact.starts_with("resets") {
        return None;
    }

    line.split_once(' ')
        .map(|(_, value)| value.trim().to_string())
        .or_else(|| {
            line.strip_prefix("Resets")
                .map(|value| value.trim().to_string())
        })
        .filter(|value| !value.is_empty())
}

fn find_line_by_compact_prefix(lines: &[String], prefix: &str) -> Option<String> {
    lines
        .iter()
        .find(|line| compact_for_matching(line).starts_with(prefix))
        .cloned()
}

fn compact_for_matching(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}
