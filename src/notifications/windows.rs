use std::io;

use super::{noop, Notification};

pub fn notify(notification: &Notification) -> io::Result<()> {
    noop::notify(notification)
}
