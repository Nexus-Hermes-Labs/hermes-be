// services/auth-service/src/infrastructure/email/lettre_email_service.rs
use std::fmt::Debug;
use async_trait::async_trait;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
};
use lettre::message::Mailbox;
use std::sync::Arc;
use crate::application::services::authentication::error::AuthApplicationError as AppError;
use crate::domain::auth_credential::EmailService;

#[derive(Debug, Clone)]
pub struct LettreEmailService {
    mailer: Arc<AsyncSmtpTransport<lettre::Tokio1Executor>>,
    from_address: Mailbox,
}

impl LettreEmailService {
    pub fn new(
        smtp_host: &str,
        smtp_port: u16,
        smtp_user: Option<String>,
        smtp_pass: Option<String>,
        from_address: &str,
        use_tls: bool,
    ) -> Result<Self, AppError> {
        use lettre::transport::smtp::client::Tls;

        let mut mailer_builder = if use_tls {
            if smtp_port == 465 {
                // Implicit TLS
                AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(smtp_host)
                    .map_err(|e| AppError::Internal(e.to_string()))?
            } else {
                // STARTTLS (usually port 587)
                AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(smtp_host)
                    .map_err(|e| AppError::Internal(e.to_string()))?
            }
        } else {
            // Plain SMTP (usually port 1025 for development)
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(smtp_host)
                .map_err(|e| AppError::Internal(e.to_string()))?
                .tls(Tls::None)
        };

        if let (Some(user), Some(pass)) = (smtp_user, smtp_pass) {
            if !user.is_empty() && !pass.is_empty() {
                let creds = Credentials::new(user, pass);
                mailer_builder = mailer_builder.credentials(creds);
            }
        }
        
        let mailer = mailer_builder.port(smtp_port).build();
        
        let from_mailbox = from_address.parse::<Mailbox>().map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(Self {
            mailer: Arc::new(mailer),
            from_address: from_mailbox,
        })
    }
}

#[async_trait]
impl EmailService for LettreEmailService {
    async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), AppError> {
        let to_mailbox = to.parse::<Mailbox>().map_err(|e| AppError::Internal(e.to_string()))?;

        // TODO: Use a proper templating engine for this
        // TODO: The base URL should come from config
        let verification_link = format!("http://localhost:8080/v1/auth/verify-email?token={}", token);
        let subject = "Welcome to Hermes! Please verify your email";
        let body = format!(
            "<h1>Welcome to Hermes!</h1><p>Please click the link below to verify your email address:</p><p><a href='{}'>Verify Email</a></p>",
            verification_link
        );

        let email = Message::builder()
            .from(self.from_address.clone())
            .to(to_mailbox)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        self.mailer
            .send(email)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send email: {}", e)))?;
        
        tracing::info!("Sent verification email to {}", to);

        Ok(())
    }
}
