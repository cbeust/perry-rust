use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, exit};
use std::str::FromStr;
use figment::Figment;
use figment::providers::{Format, Toml};
use serde::Deserialize;
use tracing::{error, info};

pub fn main() {
    // import.toml sample:
    // prod_url = "postgres://..."
    // local_url = "postgresql://localhost:5432/perry"
    let config: Config = Figment::new()
        .merge(Toml::file("import.toml"))
        .extract()
        .unwrap();
    if let Err(e) = File::open(psql(&config.postgres_dir)) {
        println!("Couldn't find psql {}: {e}", config.postgres_dir);
        exit(1);
    }

    let db = parse_jdbc_url(&config.prod_url);

    // Import production database
    let import_result = import(&config.postgres_dir, db);
    // let import_result = Some(FILE);

    // Restore local database
    match import_result {
        None => {
            error!("Something went wrong during import");
        }
        Some(file) => {
            // With user/password:  postgresql://user:pass@localhost:5432/perry
            // let default_url: String = "postgresql://localhost:5432/perry".into();
            let local_url = config.local_url;
            info!("Restoring local database {local_url} from file {file}");
            restore(&config.postgres_dir, parse_jdbc_url(&local_url), file.to_string());
        }
    }
}

fn pg_dump(pg: &str) -> String { format!("{pg}\\bin\\pg_dump.exe") }
fn psql(pg: &str) -> String { format!("{pg}\\bin\\psql.exe") }

fn import(pg: &str, db: Db) -> Option<String> {
    let file = "db.dump";
    println!("Importing database \"{}\" from {} into file {file}", db.database_name, db.host);
    match Command::new(pg_dump(pg))
        .env("PGPASSWORD", db.password)
        .arg(format!("--username={}", db.username))
        .arg("-f")
        .arg(file)
        .arg("-v")
        .arg("--no-password")
        .arg(format!("--dbname={}", db.database_name))
        .arg(format!("--host={}", db.host))
        .arg(format!("--port={}", db.port))
        .output()
    {
        Ok(_output) => {
            // println!("stdout: {:#?}", String::from_utf8_lossy(&output.stdout));
            // println!("stderr: {:#?}", String::from_utf8_lossy(&output.stderr));
            println!("Created file {file}");
            Some(file.to_string())
        }
        Err(e) => {
            println!("Error: {e}");
            None
        }
    }
}

fn restore(pg: &str, db: Db, filename: String) {
    println!("Running {} -U {} -h {} -d {} {}", psql(pg), db.username, db.host, db.database_name,
        filename);

    let mut command = Command::new(psql(pg))
        .stdin(std::process::Stdio::piped())
        .env("PGPASSWORD", db.password)
        .arg("-U")
        .arg(db.username)
        .arg("-h")
        .arg(db.host)
        .arg("-v")
        .arg("ON_ERROR_CONTINUE=on")
        .arg("-d")
        .arg("postgres")
        .spawn()
        .unwrap();

    let stdin = command.stdin.as_mut().expect("failed to open stdin");

    let commands = vec![
        "drop database perry;\n",
        "create database perry;\n",
        "\\c perry\n"
    ];

    for c in commands {
        println!("Issuing command '{c}'");
        stdin.write_all(c.as_bytes()).expect("Write to stdin");
    }
    let mut buffer: Vec<u8> = Vec::new();
    File::open(filename).unwrap().read_to_end(&mut buffer).expect("Create file");

    stdin.write_all(&buffer).expect("Write to stdin");
    drop(stdin); // Close the stdin pipe

    match command.wait_with_output() {
        Ok(output) => {
            // println!("stdout: {:#?}", String::from_utf8_lossy(&output.stdout));
            // println!("stderr: {:#?}", String::from_utf8_lossy(&output.stderr));
            println!("Database restored:\nstdout:{}\nstderr:{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            println!("Error: {e}");
        }
    }
}

fn parse_jdbc_url(url: &str) -> Db {
    let mut result = Db::default();

    let current = url.find("//").unwrap();
    let mut rest = &url[current + 2..];
    if rest.contains("@") {
        let at = rest.find('@').unwrap();
        let colon = rest.find(':').unwrap();
        result.username = rest[0..colon].to_string();
        result.password = rest[colon + 1..at].to_string();
        rest = &rest[at + 1..];
    }
    let colon = rest.find(':').unwrap();
    let slash = rest.find('/').unwrap();
    result.host = rest[0..colon].to_string();
    result.port = u16::from_str(&rest[colon + 1..slash]).unwrap();
    match rest.find('?') {
        None => {
            result.database_name = rest[slash + 1..].to_string();
        }
        Some(question) => {
            result.database_name = rest[slash + 1..question].to_string();
            rest = &rest[question + 1..];
            for pair in rest.split('&') {
                let mut kv = pair.split('=');
                match kv.next().unwrap() {
                    "username" => {
                        result.username = kv.next().unwrap().to_string();
                    }
                    "password" => {
                        result.password = kv.next().unwrap().to_string();
                    }
                    _ => {
                        println!("Ignoring {}", kv.next().unwrap())
                    }
                }
            }
        }
    }

    result
}

#[allow(unused)]
#[derive(Debug, Default, Deserialize)]
struct Db {
    host: String,
    port: u16,
    database_name: String,
    username: String,
    password: String,
}

/// Format of the file import.toml
#[allow(unused)]
#[derive(Default, Deserialize)]
struct Config {
    postgres_dir: String,
    prod_url: String,
    #[serde(default = "default_local_url")]
    local_url: String,
}

fn default_local_url() -> String {
    "postgresql://localhost:5432/perry".into()
}

#[test]
fn test_jdbc_url() {
    let data = vec![
        ("jdbc:postgres://user:pass@host.com:5432/the_db", "user", "pass"),
        ("jdbc:postgres://host.com:5432/the_db?username=user&password=pass", "user", "pass"),
        ("jdbc:postgres://host.com:5432/the_db", "", ""),
    ];
    for (url, user, pass) in data {
        let db = parse_jdbc_url(url);
        assert_eq!(db.username, user);
        assert_eq!(db.password, pass);
        assert_eq!(db.host, "host.com");
        assert_eq!(db.port, 5432);
        assert_eq!(db.database_name, "the_db");
    }

}
