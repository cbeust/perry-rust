use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use std::str::FromStr;
use tracing::{error, info};
use crate::{Args, Config};
use crate::db::Db;

pub fn run_import(args: &Args) {
    // Import production database
    let import_result = import(&args);
    // let import_result = Some(FILE);

    // Restore local database
    match import_result {
        None => {
            error!("Something went wrong during import");
        }
        Some(file) => {
            // With user/password:  postgresql://user:pass@localhost:5432/perry
            // let default_url: String = "postgresql://localhost:5432/perry".into();
            restore(args, file.to_string());
        }
    }
}

fn import(args: &Args) -> Option<String> {
    let db = Db::parse_jdbc_url(&args.config.local_url);
    let file = "db.dump";
    println!("Importing database \"{}\" from {} into file {file}", db.database_name, db.host);
    match Command::new(args.postgres.pg_dump())
        .env("PGPASSWORD", db.password.clone())
        .arg(format!("--username={}", db.username.clone()))
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

fn restore(args: &Args, filename: String) {
    let local_url = args.config.local_url.clone();
    info!("Restoring local database {local_url} from file {filename}");

    let db = Db::parse_jdbc_url(&args.config.local_url);
    println!("Running {} -U {} -h {} -d {} {}", args.postgres.psql(), db.username, db.host,
        db.database_name, filename);

    let mut command = Command::new(args.postgres.psql())
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
