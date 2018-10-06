#[macro_use]
extern crate quicli;
extern crate chrono;
extern crate postgres;

use chrono::{Local,Utc,TimeZone,DateTime};
use postgres::{Connection, TlsMode};
use quicli::prelude::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pig",
    about = "A very simple PostgreSQL data migration tool.",
    author = "Richard Anaya © 2018"
)]
struct Cli {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(
        name = "show",
        about = "Show information about the database."
    )]
    Show {
        #[structopt(subcommand)]
        command: ShowCommand,
    },

    #[structopt(name = "create", about = "Create a new migration script.")]
    Create { description: String },

    #[structopt(name = "plan", about = "Plan migration scripts.")]
    Plan {
        #[structopt(
            help = "The connection string to use. The environment variable PIG_CONNECTION_STRING can also be used.",
            short = "c"
        )]
        connection_string: Option<String>,
    },

    #[structopt(name = "apply", about = "Apply migration scripts.")]
    Apply {
        #[structopt(
            help = "The connection string to use. The environment variable PIG_CONNECTION_STRING can also be used.",
            short = "c"
        )]
        connection_string: Option<String>,
    },
}

#[derive(Debug, StructOpt)]
enum ShowCommand {
    #[structopt(name = "tables", about = "Show all tables..")]
    Tables {
        #[structopt(
            help = "The connection string to use. The environment variable PIG_CONNECTION_STRING can also be used.",
            short = "c"
        )]
        connection_string: Option<String>,
    },
    #[structopt(name = "table", about = "Show table details")]
    Table {
        #[structopt(
            help = "The connection string to use. The environment variable PIG_CONNECTION_STRING can also be used.",
            short = "c"
        )]
        connection_string: Option<String>,
        #[structopt(help = "The table to show information on.",)]
        table_name: String,
    },
}

fn get_connection_string(connection_string_opt: Option<String>) -> Result<String> {
    match connection_string_opt {
        Some(s) => return Ok(s.to_owned()),
        _ => {
            match env::var_os("PIG_CONNECTION_STRING") {
                Some(e) => return Ok(e.into_string().unwrap()),
                _ => return Err(format_err!("No connection string found, try -c <connection string> or set the environment variable PIG_CONNECTION_STRING.")),
            };
        }
    }
}

fn get_connection(connection_string_opt: Option<String>) -> Result<Connection> {
    let connection_string = get_connection_string(connection_string_opt)?;
    Ok(Connection::connect(connection_string, TlsMode::None)?)
}

fn ensure_database_info(conn: &Connection) -> Result<()> {
    let rows = &conn.query("SELECT EXISTS ( SELECT 1 FROM information_schema.tables WHERE table_name = 'pig_database_info');", &[])?;
    let info_exists: bool = rows.iter().next().unwrap().get(0);
    if !info_exists {
        conn.execute("CREATE TABLE IF NOT EXISTS pig_database_info ()", &[])?;
        conn.execute(
            "ALTER TABLE pig_database_info ADD COLUMN IF NOT EXISTS key TEXT",
            &[],
        )?;
        conn.execute(
            "ALTER TABLE pig_database_info ADD COLUMN IF NOT EXISTS value TEXT",
            &[],
        )?;
        conn.execute(
            r#"INSERT INTO pig_database_info (key,value) VALUES ('last_applied','')"#,
            &[],
        )?;
    }
    Ok(())
}

fn get_pig_key_value(conn: &Connection, key: String) -> Result<String> {
    let rows = &conn.query("SELECT value FROM pig_database_info WHERE key=$1", &[&key])?;
    Ok(rows.iter().next().unwrap().get(0))
}

fn get_last_applied(conn: &Connection) -> Result<DateTime<Utc>>{
    let last_applied = get_pig_key_value(conn,"last_applied".to_owned())?;
    let mut last_applied_date = Utc.datetime_from_str(&"19700101000000", "%Y%m%d%H%M%S")?;
    if !last_applied.is_empty() {
        let date_str:String = last_applied.chars().take(14).collect();
        last_applied_date = Utc.datetime_from_str(&date_str, "%Y%m%d%H%M%S")?
    }
    Ok(last_applied_date)
}

fn apply(connection_string_opt: Option<String>) -> Result<()> {
    let conn = get_connection(connection_string_opt)?;
    ensure_database_info(&conn)?;
    println!("applying ");
    Ok(())
}

fn show_tables(connection_string_opt: Option<String>) -> Result<()> {
    let conn = get_connection(connection_string_opt)?;
    for row in &conn.query("SELECT table_name FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE'", &[]).unwrap() {
        let table_name:String = row.get(0);
        println!("{}",table_name);
    };
    Ok(())
}

fn show_table(connection_string_opt: Option<String>, table_name: String) -> Result<()> {
    let conn = get_connection(connection_string_opt)?;
    for row in &conn.query(&format!("SELECT column_name, data_type FROM information_schema.columns WHERE table_name   = '{}'",table_name), &[]).unwrap() {
        let column_name:String = row.get(0);
        let data_type:String = row.get(1);
        println!("{} : {}",column_name, data_type);
    };
    Ok(())
}

fn create(description: String) -> Result<()> {
    let date = Local::now();
    let mut file_description = description.to_lowercase().replace(" ", "_");
    file_description.truncate(30);
    let filename = format!("{}_{}.sql", date.format("%Y%m%d%H%M%S"), file_description);
    println!("{}", filename);
    let mut file = File::create(filename)?;
    file.write_all(format!("# {}\n\n", description).as_bytes())?;
    Ok(())
}

fn plan(connection_string_opt: Option<String>) -> Result<()> {
    let conn = get_connection(connection_string_opt)?;
    ensure_database_info(&conn)?;
    let last_applied = get_last_applied(&conn)?;
    println!("{:?}",last_applied);
    Ok(())
}

main!(|args: Cli| match args.command {
    Command::Apply { connection_string } => apply(connection_string)?,
    Command::Create { description } => create(description)?,
    Command::Show { command } => match command {
        ShowCommand::Tables { connection_string } => show_tables(connection_string)?,
        ShowCommand::Table {
            connection_string,
            table_name,
        } => show_table(connection_string, table_name)?,
    },
    Command::Plan { connection_string } => plan(connection_string)?,
});
