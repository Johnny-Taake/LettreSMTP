use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};
use tracing::error;

use crate::config::CONFIG;
use crate::utils::log_email_to_file;

pub fn send_email(
    recipient: &str,
    subject: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let from: Mailbox = CONFIG.smtp_user.parse()?;
    let to: Mailbox = recipient.parse()?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .body(body.to_string())?;

    let creds = Credentials::new(CONFIG.smtp_user.clone(), CONFIG.smtp_password.clone());
    let mailer = SmtpTransport::relay(&CONFIG.smtp_server)?
        .port(CONFIG.smtp_port)
        .credentials(creds)
        .authentication(vec![Mechanism::Login])
        .build();

    let result = mailer.send(&email);

    let success = result.is_ok();
    if let Err(e) = log_email_to_file(recipient, subject, body, success) {
        error!("Warning: Failed to write to log file: {}", e);
    }

    result?;
    Ok(())
}
