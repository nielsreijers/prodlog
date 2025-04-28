use chrono::{DateTime, Utc};
use super::CaptureState;


pub mod obsidian;
pub mod json;

pub trait Sink {
    fn add_entry(&mut self, capture: &CaptureState, exit_code: i32, end_time: DateTime<Utc>) -> Result<(), std::io::Error>;
}

fn get_short_command(cmd: &str) -> String {
    let mut words = cmd.split_whitespace();
    let first_word = words.next().unwrap_or("");
    if first_word == "sudo" {
        format!("{} {}", first_word, words.next().unwrap_or(""))
    } else {
        first_word.to_string()
    }
}
fn get_formatted_time_long(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string()
}
fn get_formatted_time_short(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M").to_string()
}

// std::fs::create_dir_all(prodlog_dir.join(format!("prodlog_output/{}", host)))?;
// std::fs::create_dir_all(prodlog_dir.join("prodlog_output/all-hosts"))?;

// let start_time = Utc::now();
// let formatted_start_long = start_time.format("%Y-%m-%d %H:%M:%S%.3f UTC");
// let log_filename_by_host = Self::get_by_host_log_filename(start_time, host, cmd);
// let log_filename_all_hosts = Self::get_all_hosts_log_filename(start_time, host, cmd);
// let mut log_by_host = File::create(prodlog_dir.join(log_filename_by_host.clone()))?;
// let mut log_all_hosts = File::create(prodlog_dir.join(log_filename_all_hosts.clone()))?;

// let header = format!(
//     "Host:     {host}\n\
//     Start:    {formatted_start_long}\n\
//     Command:  {cmd}\n\
//     Output:\n\
//     ```\n\
//     ");
// log_by_host.write_all(header.as_bytes())?;
// log_all_hosts.write_all(header.as_bytes())?;









// std::fs::create_dir_all(prodlog_dir).unwrap();

// let end_time = Utc::now();
// let duration_ms = end_time.signed_duration_since(state.start_time).num_milliseconds() as u64;
// let formatted_start_short = state.start_time.format("%Y-%m-%d %H:%M");
// let formatted_start_long = state.start_time.format("%Y-%m-%d %H:%M:%S%.3f UTC");
// let formatted_end_long = end_time.format("%Y-%m-%d %H:%M:%S%.3f UTC");
// let host = &state.host;
// let cmd_short = Self::get_short_command(&state.cmd);
// let cmd_long = &state.cmd;
// let log_filename = &state.log_filename_by_host;

// let entry = format!(
//     "\n## {formatted_start_short} on {host}: {cmd_short}\n\
//     ```\n\
//     Host:     {host}\n\
//     Start:    {formatted_start_long}\n\
//     Command:  {cmd_long}\n\
//     End:      {formatted_end_long}\n\
//     Duration: {duration_ms}ms\n\
//     ExitCode: {exit_code}\n\
//     ```\n\
//     Output:   [[{log_filename}]]\n\
//     \n\
//     ---\n\
//     ");

// std::fs::OpenOptions::new()
//     .create(true)
//     .append(true)
//     .open(prodlog_dir.join("prodlog.md"))?
//     .write_all(entry.as_bytes())?;

// // Log to JSON file for webui
// let json_path = prodlog_dir.join("prodlog.json");
// let mut prodlog_data = if let Ok(content) = fs::read_to_string(&json_path) {
//     serde_json::from_str(&content).unwrap_or(ProdlogData { entries: Vec::new() })
// } else {
//     ProdlogData { entries: Vec::new() }
// };
// prodlog_data.entries.push(ProdlogEntry {
//     host: host.to_string(),
//     start_time: state.start_time.to_rfc3339(),
//     end_time: end_time.to_rfc3339(),
//     duration_ms,
//     command: cmd_long.to_string(),
//     log_filename: log_filename.to_string(),
//     prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
//     exit_code,
// });
// fs::write(&json_path, serde_json::to_string_pretty(&prodlog_data)?)?;

// let footer = format!(
//     "```\n\
//     End:      {formatted_end_long}\n\
//     Duration: {duration_ms}ms\n\
//     ExitCode: {exit_code}\n");
// state.log_by_host.write_all(footer.as_bytes())?;
// state.log_all_hosts.write_all(footer.as_bytes())?;
// state.log_by_host.flush()?;
// state.log_all_hosts.flush()?;