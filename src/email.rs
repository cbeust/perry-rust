use askama::Template;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use tracing::{error, info};
use tracing::log::warn;
use crate::config::Config;
use crate::db::Db;
use crate::entities::Cycle;
use crate::errors::Error::{FetchingCycles, Unknown};
use crate::errors::PrResult;
use crate::perrypedia::PerryPedia;
use crate::url::Urls;


#[derive(Template)]
#[template(path = "email-summary.html")]
struct SendEmailTemplate {
    cycle_name: String,
    cover_url: String,
    english_title: String,
    german_title: String,
    heft_author: String,
    summary_author_name: String,
    summary_text: String,
    summary_url: String,
}

pub struct Email;

impl Email {
    pub async fn create_email_service(config: &Config) -> Box<dyn EmailService> {
        if config.send_emails || config.is_heroku {
            match (config.email_username.clone(), config.email_password.clone()) {
                (Some(u), Some(p)) => {
                    info!("Sending real emails");
                    Box::new(EmailProduction::builder().username(u).password(p).build())
                }
                _ => {
                    if config.is_heroku {
                        error!("On Heroku but no username/password, using mock EmailService");
                    } else {
                        warn!("Asked to send emails but no username/password, using mock EmailService");
                    }
                    Box::new(EmailMock)
                }
            }
        } else {
            info!("Using mock EmailService");
            Box::new(EmailMock)
        }
    }

    pub async fn create_email_content_for_summary(db: &Box<dyn Db>, book_number: u32,
            host: String)
        -> PrResult<String>
    {
        let (book, summary, cycle_number, cover_url) = tokio::join!(
            db.find_book(book_number),
            db.find_summary(book_number),
            db.find_cycle_by_book(book_number),
            PerryPedia::find_cover_url(book_number),
        );

        match cycle_number {
            None => { Err(FetchingCycles("".into())) }
            Some(cycle) => {
                let cycle_name = cycle.english_title;
                match (book, summary) {
                    (Some(book), Some(summary)) => {
                        let template = SendEmailTemplate {
                            cycle_name,
                            cover_url: cover_url.unwrap_or("".to_string()),
                            english_title: summary.english_title,
                            german_title: book.title,
                            heft_author: book.author,
                            summary_author_name: summary.author_name,
                            summary_text: summary.summary,
                            summary_url: format!("{}{}", host, Urls::summary(book_number as i32)),
                        };
                        let content = template.render().unwrap();
                        Ok(content.into())
                    }
                    _ => {
                        Err(Unknown)
                    }
                }
            }
        }
    }
}

pub trait EmailService: Send + Sync {
    fn send_email(&self, to: &str, subject: &str, body: &str);
}

pub struct EmailMock;

impl EmailService for EmailMock {
    fn send_email(&self, to: &str, subject: &str, body: &str) {
        info!("Would have sent email to {to} with subject '{subject}'");
    }
}

#[derive(bon::Builder)]
pub struct EmailProduction {
    username: String,
    password: String,
}

impl EmailService for EmailProduction {
    fn send_email(&self, to: &str, subject: &str, body: &str) {
        let email = Message::builder()
            .from("Perry Rhodan Summaries <perry.summary@gmail.com>".parse().unwrap())
            .reply_to("Nobody <nobody@nobody.com>".parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.to_string())
            // .singlepart(SinglePart::html(body))
            .unwrap();

        let creds = Credentials::new(self.username.clone(), self.password.clone());

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => info!("Email sent successfully to {to}!"),
            Err(e) => error!("Could not send email to {to}: {e:?}"),
        }

    }
}