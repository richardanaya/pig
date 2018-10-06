#[macro_use]
extern crate quicli;
use quicli::prelude::*;


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
    #[structopt(name = "apply")]
    Apply {
        #[structopt(help = "The connection string to apply with. The environment variable PIG_CONNECTION_STRING can also be used.", short = "c")]
        connection_string: Option<String>
    },
}

fn apply(connection_string_opt:Option<String>)-> Result<()> {
    let connection_string = match connection_string_opt {
        Some(s) => s,
        _ => {
            return Err(format_err!("No connection string found, try -c <connection string> or set the environment variable PIG_CONNECTION_STRING."))
        }
    };
    println!("apply");
    Ok(())
}

main!(|args: Cli| match args.command {
    Command::Apply { connection_string} => apply(connection_string)?
});
