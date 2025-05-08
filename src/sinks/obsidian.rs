use std::{fs::File, io::Write, path::PathBuf};
use chrono::Duration;
use crate::sinks::get_short_command;
use crate::model::CaptureV2_2;
use super::{get_formatted_time_long, get_formatted_time_short, Sink};

pub struct ObsidianSink {
    prodlog_dir: PathBuf,
}

impl ObsidianSink {
    pub fn new(prodlog_dir: &PathBuf) -> Self {
        let prodlog_dir = prodlog_dir.clone();
        Self { prodlog_dir }
    }
}

fn get_output_log_filename (capture: &CaptureV2_2) -> String {
    let formatted_time = capture.start_time.format("%Y%m%d_%H%M%S").to_string();
    let short_cmd = get_short_command(&capture.cmd).replace(" ", "_");
    format!("prodlog_output/{}/{}-{}.md", capture.host, formatted_time, short_cmd)
}

impl Sink for ObsidianSink {
    fn add_entry(&mut self, capture: &CaptureV2_2) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(self.prodlog_dir.join(format!("prodlog_output/{}", capture.host)))?;
        let output_filename = get_output_log_filename(capture);
        let mut log_by_host = File::create(self.prodlog_dir.join(output_filename.clone()))?;

        let host = &capture.host;
        let formatted_start_long = get_formatted_time_long(capture.start_time);
        let formatted_start_short = get_formatted_time_short(capture.start_time);
        let end_time = capture.start_time + Duration::milliseconds(capture.duration_ms as i64);
        let formatted_end_long = get_formatted_time_long(end_time);
        let duration_ms = capture.duration_ms;
        let exit_code = capture.exit_code;
        let cmd = &capture.cmd;
        let cmd_short = get_short_command(cmd);

        // Write command output to a file
        let header = format!(
            "Host:     {host}\n\
            Start:    {formatted_start_long}\n\
            Command:  {cmd}\n\
            Output:\n\
            ```\n\
            ");
        let footer = format!(
            "```\n\
            End:      {formatted_end_long}\n\
            Duration: {duration_ms}ms\n\
            ExitCode: {exit_code}\n");
        log_by_host.write_all(header.as_bytes())?;
        log_by_host.write_all(&capture.captured_output)?;
        log_by_host.write_all(footer.as_bytes())?;
        log_by_host.flush()?;
        
        // Write entry to log file
        let entry = format!(
            "\n## {formatted_start_short} on {host}: {cmd_short}\n\
            ```\n\
            Host:     {host}\n\
            Start:    {formatted_start_long}\n\
            Command:  {cmd}\n\
            End:      {formatted_end_long}\n\
            Duration: {duration_ms}ms\n\
            ExitCode: {exit_code}\n\
            ```\n\
            Output:   [[{output_filename}]]\n\
            \n\
            ---\n\
            ");

        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.prodlog_dir.join("prodlog.md"))?
            .write_all(entry.as_bytes())?;

        Ok(())
    }
}
