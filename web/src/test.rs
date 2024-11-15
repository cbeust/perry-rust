#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::db::{Db, MockDb};
    use crate::email::Email;
    use crate::pages::cycles::index;
    use crate::perrypedia::PerryPedia;
    use crate::PerryState;
    use actix_web::web::Data;
    use actix_web::{test, App};
    use std::sync::Arc;

    async fn create_state(db: Box<dyn Db>) -> PerryState {
        let config = Config::default();
        PerryState {
            app_name: "Perry Test".into(),
            config: config.clone(),
            db: Arc::new(db),
            email_service: Arc::new(Email::create_email_service(&config).await),
            perry_pedia: PerryPedia::new(),
        }
    }

    #[actix_web::test]
    async fn test_index_get() {
        let mut db = MockDb::new();
        db.expect_fetch_cycles()
            .returning(|| Ok(Vec::new()));
        db.expect_fetch_most_recent_summaries()
            .returning(|| Vec::new());
        db.expect_fetch_summary_count()
            .returning(|| 100);
        db.expect_fetch_book_count()
            .returning(|| 1000);
        let state = create_state(Box::new(db)).await;
        let app = test::init_service(App::new()
            .service(index)
            .app_data(Data::new(state.clone()))
        ).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_and_read_body(&app, req).await;
        let string = std::str::from_utf8(&resp).unwrap();
        assert!(string.contains("Total written summaries: 100 (10 %)"));
    }
}