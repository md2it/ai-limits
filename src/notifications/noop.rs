use std::io;

use super::Notification;

pub fn notify(_notification: &Notification) -> io::Result<()> {
    Ok(())
}
