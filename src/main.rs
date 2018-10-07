#[macro_use]
extern crate quicli;
extern crate chrono;
extern crate colored;
extern crate glob;
extern crate postgres;

use chrono::{DateTime, Local, TimeZone, Utc};
use colored::*;
use glob::glob;
use postgres::{Connection, TlsMode};
use quicli::prelude::*;
use std::cmp::Ordering;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
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

    #[structopt(
        name = "modify",
        about = "Modify the current migration."
    )]
    Modify {
        #[structopt(subcommand)]
        command: ModifyCommand,
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
    #[structopt(name = "tables", about = "Show all tables.")]
    Tables {
        #[structopt(
            help = "The connection string to use. The environment variable PIG_CONNECTION_STRING can also be used.",
            short = "c"
        )]
        connection_string: Option<String>,
    },
    #[structopt(name = "table", about = "Show table details.")]
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

#[derive(Debug, StructOpt)]
enum ModifyCommand {
    #[structopt(name = "add-table", about = "Append an add table command to current migration.")]
    AddTable {
        #[structopt(
            help = "The connection string to use. The environment variable PIG_CONNECTION_STRING can also be used.",
            short = "c"
        )]
        connection_string: Option<String>,
        table_name: String,
    },
    #[structopt(name = "add-column", about = "Append an add table column command to current migration.")]
    AddColumn {
        #[structopt(
            help = "The connection string to use. The environment variable PIG_CONNECTION_STRING can also be used.",
            short = "c"
        )]
        connection_string: Option<String>,
        table_name: String,
        column_name: String,
        type_name: String,
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

fn get_last_applied(conn: &Connection) -> Result<DateTime<Utc>> {
    let last_applied = get_pig_key_value(conn, "last_applied".to_owned())?;
    let mut last_applied_date = Utc.datetime_from_str(&"19700101000000", "%Y%m%d%H%M%S")?;
    if !last_applied.is_empty() {
        let date_str: String = last_applied.chars().take(14).collect();
        last_applied_date = Utc.datetime_from_str(&date_str, "%Y%m%d%H%M%S")?
    }
    Ok(last_applied_date)
}

fn apply(connection_string_opt: Option<String>) -> Result<()> {
    let conn = get_connection(connection_string_opt)?;
    ensure_database_info(&conn)?;
    let last_applied = get_last_applied(&conn)?;
    let mut project_files = get_project_files().unwrap();
    project_files.sort();
    project_files.retain(|ref i| i.date > last_applied);
    let mut sql = String::new();
    for project_file in project_files.iter() {
        let mut file = File::open(&project_file.file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        sql.push_str(&contents);
        sql.push_str("\n");
    }
    println!(
        "{}",
        format!("Applying {} files:", project_files.len()).green()
    );
    if project_files.len() > 0 {
        println!("Beginning transaction");
        let trans = conn.transaction()?;
        let commands: Vec<&str> = sql.split(';').collect();
        for command in commands.iter() {
            println!("{}",command);
            trans.execute(command, &[])?;
        }
        let last_date: String = project_files[project_files.len() - 1]
            .date
            .format("%Y%m%d%H%M%S")
            .to_string();
        trans.execute(
            "UPDATE pig_database_info SET value=$1 WHERE key='last_applied';",
            &[&last_date],
        )?;
        trans.commit()?;
        println!("Transaction committed");
    }
    println!("{}", "Complete".green());
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
    file.write_all(format!("-- {}\n\n", description).as_bytes())?;
    Ok(())
}

fn plan(connection_string_opt: Option<String>) -> Result<()> {
    let conn = get_connection(connection_string_opt)?;
    ensure_database_info(&conn)?;
    let last_applied = get_last_applied(&conn)?;
    let mut project_files = get_project_files().unwrap();
    project_files.sort();
    project_files.retain(|ref i| i.date > last_applied);
    let mut sql = String::new();
    for project_file in project_files.iter() {
        let mut file = File::open(&project_file.file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        sql.push_str(&contents);
        sql.push_str("\n");
    }
    if project_files.len() != 0 {
        println!("{}", "SQL:".green());
        println!("{}", sql);
    }
    println!(
        "{}",
        format!("Plan: {} to apply", project_files.len()).green()
    );
    for project_file in project_files.iter() {
        println!("    {} {}", "+".green(), project_file.file.green());
    }
    Ok(())
}

fn append_to_latest_migration(conn:&Connection, sql:String) -> Result<()> {
    let latest_project_file = get_latest_project_file()?;
    match latest_project_file {
        Some(file_name) => {
            if !is_current_migration_unused(conn,&file_name)? {
                println!("Current migration was already deployed. Create a new one.");
                return Ok(());
            }


            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(file_name)
                .unwrap();

            file.write_all(format!("\n{}",sql).as_bytes())?;
        },
        None => println!("There are no migrations. Create one first.")
    }
    Ok(())
}

fn is_current_migration_unused(conn:&Connection, file_name:&String) -> Result<bool>{
    let last_applied = get_last_applied(conn)?;
    let date_prefix: String = file_name.chars().take(14).collect();
    match Utc.datetime_from_str(&date_prefix, "%Y%m%d%H%M%S") {
        Ok(date) => {
            match date.cmp(&last_applied) {
                Ordering::Greater => return  Ok(true),
                _ => return Ok(false)
            }
        }
        _ => println!("Unknown SQL file: {:?}", file_name),
    }
    Err(format_err!("Latest migration has invalid prefix."))
}

fn add_table(connection_string_opt: Option<String>,table_name: String) -> Result<()> {
    let conn = get_connection(connection_string_opt)?;
    let sql = format!("CREATE TABLE IF NOT EXISTS {} ();", table_name);
    append_to_latest_migration(&conn,sql)?;
    Ok(())
}

fn add_table_column(connection_string_opt: Option<String>,table_name: String, column_name: String, type_name: String) -> Result<()> {
    let conn = get_connection(connection_string_opt)?;
    let sql = format!("
IF NOT EXISTS( SELECT NULL
        FROM INFORMATION_SCHEMA.COLUMNS
       WHERE table_name = '{}'
         AND column_name = '{}')  THEN
         ALTER TABLE `{}` ADD `{}` {};
END IF;
    ", table_name, column_name, table_name, column_name, type_name);
    append_to_latest_migration(&conn,sql)?;
    Ok(())
}

#[derive(Debug, Eq)]
struct ProjectFile {
    file: String,
    date: DateTime<Utc>,
}

impl Ord for ProjectFile {
    fn cmp(&self, other: &ProjectFile) -> Ordering {
        self.date.cmp(&other.date)
    }
}

impl PartialOrd for ProjectFile {
    fn partial_cmp(&self, other: &ProjectFile) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ProjectFile {
    fn eq(&self, other: &ProjectFile) -> bool {
        self.date == other.date
    }
}

fn get_project_files() -> Result<Vec<ProjectFile>> {
    let mut files = Vec::<ProjectFile>::new();
    for entry in glob("*.sql").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    let file_name: String = path.file_name().unwrap().to_str().unwrap().to_owned();
                    let date_prefix: String = file_name.chars().take(14).collect();
                    match Utc.datetime_from_str(&date_prefix, "%Y%m%d%H%M%S") {
                        Ok(date) => {
                            files.push(ProjectFile {
                                file: file_name,
                                date: date,
                            });
                        }
                        _ => println!("Unknown SQL file: {:?}", file_name),
                    }
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(files)
}

fn get_latest_project_file() -> Result<Option<String>> {
    let project_files = get_project_files()?;
    if project_files.len() == 0 {
        return Ok(None);
    }
    Ok(Some(project_files[project_files.len()-1].file.clone()))
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
    Command::Modify { command } => match command {
        ModifyCommand::AddTable { connection_string,table_name } => add_table(connection_string,table_name)?,
        ModifyCommand::AddColumn {
            connection_string,
            table_name,
            column_name,
            type_name
        } => add_table_column(connection_string,table_name, column_name, type_name)?,
    },
    Command::Plan { connection_string } => plan(connection_string)?,
});
