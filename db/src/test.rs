use crate::db::Db;

#[test]
fn test_jdbc_url() {
    let data = vec![
        ("jdbc:postgres://user:pass@host.com:5432/the_db", "user", "pass"),
        ("jdbc:postgres://host.com:5432/the_db?username=user&password=pass", "user", "pass"),
        ("jdbc:postgres://host.com:5432/the_db", "", ""),
    ];
    for (url, user, pass) in data {
        let db = Db::parse_jdbc_url(url);
        assert_eq!(db.username, user);
        assert_eq!(db.password, pass);
        assert_eq!(db.host, "host.com");
        assert_eq!(db.port, 5432);
        assert_eq!(db.database_name, "the_db");
    }

}
