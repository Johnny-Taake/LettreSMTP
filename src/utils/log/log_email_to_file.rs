use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;

pub fn log_email_to_file(
    recipient: &str,
    subject: &str,
    _message: &str,
    success: bool,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("email_sent.log")?;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let status = if success { "SUCCESS" } else { "FAILED" };

    writeln!(
        file,
        "[{}] {} | To: {} | Subject: {}",
        timestamp, status, recipient, subject
    )?;

    Ok(())
}
