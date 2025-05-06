use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;
use crate::model::CaptureV2_2;


pub mod obsidian;
pub mod json;
pub mod sqlite;

pub trait Sink {
    fn add_entry(&mut self, capture: &CaptureV2_2) -> Result<(), std::io::Error>;
}

#[derive(Deserialize, Debug, Default)]
pub struct Filters {
    pub date: Option<String>,
    pub host: Option<String>,
    pub command: Option<String>,
    pub output: Option<String>,
}

pub trait UiSink: Send + Sync {
    fn get_entries(&self, filters: &Filters) -> Result<Vec<CaptureV2_2>, std::io::Error>;
    fn get_entry_by_id(&self, uuid: Uuid) -> Result<Option<CaptureV2_2>, std::io::Error>;
}

fn get_short_command(cmd: &str) -> String {
    let mut words = cmd.split_whitespace()
        .map(|w| w.rsplit_once('/').map(|(_, last)| last).unwrap_or(w));
    let first = words.next().unwrap_or("");
    let second = words.next().unwrap_or("");
    if first == "sudo" && !second.is_empty() {
        format!("sudo {}", second)
    } else {
        first.to_string()
    }
}

fn get_formatted_time_long(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string()
}

fn get_formatted_time_short(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_short_command_basic() {
        assert_eq!(get_short_command("/sbin/sudo /bin/nano /etc/fstab"), "sudo nano");
        assert_eq!(get_short_command("/usr/bin/python3 /tmp/script.py"), "python3");
        assert_eq!(get_short_command("/bin/ls -l /home"), "ls");
        assert_eq!(get_short_command("sudo /usr/bin/vim /etc/hosts"), "sudo vim");
        assert_eq!(get_short_command("nano /etc/hosts"), "nano");
        assert_eq!(get_short_command("/bin/echo hello world"), "echo");
    }

    #[test]
    fn test_get_short_command_empty() {
        assert_eq!(get_short_command("").as_str(), "");
    }
}
