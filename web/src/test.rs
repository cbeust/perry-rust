#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use actix_web::{http::header::ContentType, test, App};
    use actix_web::web::Data;
    use crate::config::Config;
    use crate::db::{Db, DbInMemory, MockDb};
    use crate::db::__mock_MockDb_Db::__fetch_cycles::Expectation;
    use crate::email::Email;
    use crate::pages::cycles::index;
    use crate::perrypedia::PerryPedia;
    use crate::PerryState;

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
            .returning(|| 0);
        db.expect_fetch_book_count()
            .returning(|| 0);
        let state = create_state(Box::new(db)).await;
        let app = test::init_service(App::new()
            .service(index)
            .app_data(Data::new(state.clone()))
        ).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}