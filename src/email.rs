use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use tracing::{error, info};
use crate::config::Config;

pub async fn create_email_service(config: &Config) -> Box<dyn EmailService> {
    if config.send_emails {
        match (config.email_username.clone(), config.email_password.clone()) {
            (Some(u), Some(p)) => {
                info!("Sending real emails");
                Box::new(EmailProduction::builder().username(u).password(p).build())
            }
            _ => {
                error!("Email service was requested but couldn't find username/password, using mock");
                Box::new(EmailMock)
            }
        }
    } else {
        info!("Using mock EmailService");
        Box::new(EmailMock)
    }
}

pub trait EmailService: Send + Sync {
    fn send_email(&self, subject: String);
}

pub struct EmailMock;

impl EmailService for EmailMock {
    fn send_email(&self, subject: String) {
        info!("Would have sent email with subject '{subject}'");
    }
}

#[derive(bon::Builder)]
pub struct EmailProduction {
    username: String,
    password: String,
}

impl EmailService for EmailProduction {
    fn send_email(&self, subject: String) {
        let email = Message::builder()
            .from("Perry Rhodan Summaries <perry.summary@gmail.com>".parse().unwrap())
            .reply_to("Nobody <nobody@nobody.com>".parse().unwrap())
            .to("cbeust@gmail.com".parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(String::from("Ad astra!"))
            .unwrap();

        let creds = Credentials::new(self.username.clone(), self.password.clone());

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => panic!("Could not send email: {e:?}"),
        }

    }
}