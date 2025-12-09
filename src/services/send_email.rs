use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};
use tracing::{error, debug, warn};

use crate::config::CONFIG;
use crate::utils::log_email_to_file;
use crate::utils::mask_string::mask_email;

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

    debug!("Attempting to send email to: {} with subject: {}", mask_email(recipient), subject);

    let result = mailer.send(&email);

    let success = result.is_ok();
    if let Err(e) = log_email_to_file(recipient, subject, body, success) {
        error!("Warning: Failed to write to log file: {}", e);
    }

    match result {
        Ok(_) => {
            debug!("Email sent successfully to: {}", mask_email(recipient));
            Ok(())
        }
        Err(e) => {
            error!(
                "SMTP send failed for recipient: {}, server: {}, error: {}",
                mask_email(recipient),
                CONFIG.smtp_server,
                e
            );

            if e.to_string().to_lowercase().contains("authentication") ||
               e.to_string().to_lowercase().contains("login") ||
               e.to_string().to_lowercase().contains("credentials") {
                warn!(
                    "Authentication failed for SMTP server: {} with user: {}, please check credentials",
                    CONFIG.smtp_server,
                    mask_email(&CONFIG.smtp_user)
                );
            }

            Err(Box::new(e))
        }
    }
}
