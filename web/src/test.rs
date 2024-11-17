

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::db::{Db, MockDb};
    use crate::email::Email;
    use crate::entities::{Book, Cycle, Summary};
    use crate::errors::PrResult;
    use crate::pages::cycles::index;
    use crate::perrypedia::CoverFinder;
    use crate::{init_logging, PerryState};
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::web::Data;
    use actix_web::{test, App, Error};
    use async_trait::async_trait;
    use figment::providers::{Format, Json};
    use figment::Figment;
    use serde::Deserialize;
    use std::process::exit;
    use std::sync::Arc;

    #[derive(Default, Deserialize)]
    pub struct Content {
        pub books: Vec<Book>,
        pub cycles: Vec<Cycle>,
        pub summaries: Vec<Summary>,
    }

    struct DbTest {
        content: Content,
    }

    impl Default for DbTest {
        fn default() -> Self {
            Self { content: Self::init_content() }
        }
    }

    impl DbTest {
        fn init_content() -> Content {
            match Figment::new()
                .merge(Json::file("web/src/test.json"))
                .extract()
            {
                Ok(content) => { content }
                Err(e) => {
                    println!("Couldn't parse the config: {e}");
                    exit(1);
                }
            }
        }
    }

    #[async_trait]
    impl Db for DbTest {
        async fn fetch_cycles(&self) -> PrResult<Vec<Cycle>> {
            Ok(self.content.cycles.clone())
        }

        async fn fetch_summary_count(&self) -> u16 {
            self.content.summaries.len() as u16
        }

        async fn fetch_book_count(&self) -> u16 {
            self.content.books.len() as u16
        }

        async fn fetch_most_recent_summaries(&self) -> Vec<Summary> {
            self.content.summaries.clone()
        }
    }

    struct CoverFinderTest;
    impl CoverFinder for CoverFinderTest {}

    async fn create_state(db: Box<dyn Db>) -> PerryState {
        let config = Config::default();
        PerryState {
            app_name: "Perry Test".into(),
            config: config.clone(),
            db: Arc::new(db),
            email_service: Arc::new(Email::create_email_service(&config).await),
            cover_finder: Arc::new(Box::new(CoverFinderTest{})),
        }
    }

    async fn setup() -> impl Service<actix_http::Request,
        Response = ServiceResponse, Error = Error>
    {
        init_logging().sqlx(false).actix(true).call();
        let db = DbTest::default();
        let state = create_state(Box::new(db)).await;
        let result = test::init_service(App::new()
            .service(index)
            .app_data(Data::new(state.clone()))
        ).await;

        result
    }

    #[actix_web::test]
    async fn test_index_get() {
        let app = setup().await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_and_read_body(&app, req).await;
        let string = std::str::from_utf8(&resp).unwrap();
        assert!(string.contains("Total written summaries: 2 (100 %)"));
    }
}