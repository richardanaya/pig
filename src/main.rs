#[macro_use]
extern crate quicli;
use quicli::prelude::*;
use std::env;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "🐷 Pig",
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
        #[structopt(
            help = "The connection string to use. The environment variable PIG_CONNECTION_STRING can also be used.",
            short = "c"
        )]
        connection_string: Option<String>,
    },

    #[structopt(name = "generate", about = "Generate migration scripts.")]
    Generate {
        #[structopt(
            help = "The connection string to use. The environment variable PIG_CONNECTION_STRING can also be used.",
            short = "c"
        )]
        connection_string: Option<String>,
    },

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

fn get_connection_string(connection_string_opt: Option<String>) -> Result<String> {
    match connection_string_opt {
        Some(s) => return Ok(s.to_owned()),
        _ => {
            let connection_string_env_opt = env::var_os("PIG_CONNECTION_STRING");
            let connection_string_env = match connection_string_env_opt {
                Some(e) => return Ok(e.into_string().unwrap()),
                _ => return Err(format_err!("No connection string found, try -c <connection string> or set the environment variable PIG_CONNECTION_STRING.")),
            };
        }
    }
}

fn apply(connection_string_opt: Option<String>) -> Result<()> {
    let connection_string = get_connection_string(connection_string_opt)?;
    println!("applying ");
    Ok(())
}

fn show(connection_string_opt: Option<String>) -> Result<()> {
    let connection_string = get_connection_string(connection_string_opt)?;
    println!("show ");
    Ok(())
}

fn generate(connection_string_opt: Option<String>) -> Result<()> {
    let connection_string = get_connection_string(connection_string_opt)?;
    println!("generate ");
    Ok(())
}

fn plan(connection_string_opt: Option<String>) -> Result<()> {
    let connection_string = get_connection_string(connection_string_opt)?;
    println!("generate ");
    Ok(())
}

main!(|args: Cli| match args.command {
    Command::Apply { connection_string } => apply(connection_string)?,
    Command::Generate { connection_string } => generate(connection_string)?,
    Command::Show { connection_string } => show(connection_string)?,
    Command::Plan { connection_string } => plan(connection_string)?,
});
