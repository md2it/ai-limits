use std::io;
use std::process::Command;

use super::Notification;

pub fn notify(notification: &Notification) -> io::Result<()> {
    let script = format!(
        "display notification {} with title {} subtitle {}",
        apple_script_string(&notification.message),
        apple_script_string(&notification.title),
        apple_script_string(&notification.subtitle)
    );

    let output = Command::new("osascript").arg("-e").arg(script).output()?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(io::Error::new(
        io::ErrorKind::Other,
        if stderr.is_empty() {
            "osascript failed to show notification".to_string()
        } else {
            stderr
        },
    ))
}

fn apple_script_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ");

    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_apple_script_strings() {
        assert_eq!(
            apple_script_string("a \"b\" \\ c"),
            "\"a \\\"b\\\" \\\\ c\""
        );
    }
}
