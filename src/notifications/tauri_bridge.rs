use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use super::{Notification, NotificationDelivery};

pub const NOTIFICATION_BRIDGE_ADDR: &str = "127.0.0.1:39745";

pub struct TauriNotificationBridge;

impl NotificationDelivery for TauriNotificationBridge {
    fn deliver(&self, notification: &Notification) -> io::Result<()> {
        let Ok(addr) = NOTIFICATION_BRIDGE_ADDR.parse::<SocketAddr>() else {
            return Ok(());
        };

        let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(150)) else {
            return Ok(());
        };

        stream.set_write_timeout(Some(Duration::from_millis(150)))?;
        let payload = serde_json::to_vec(notification)?;
        stream.write_all(&payload)?;
        stream.write_all(b"\n")?;
        Ok(())
    }
}
