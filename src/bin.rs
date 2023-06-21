#[allow(unused)]
use clap::Parser;
use clap::Subcommand;
use service::run;
use service::send;

use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Send file to target
    Send {
        /// Target node IP address to send the file to
        #[arg(short, long, value_parser = clap::value_parser!(IpAddr))]
        destination: IpAddr,
        /// Source file to send
        #[arg(short, long)]
        source: String,
        /// Target file path
        #[arg(short, long)]
        target: String,
        /// Target port number
        #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..))]
        port: u16,
    },
    Run {
        /// Target node IP address to send the file to
        #[arg(short, long, value_parser = clap::value_parser!(IpAddr))]
        destination: IpAddr,
        /// Remote command to run
        #[arg(short, long)]
        command: String,
        /// Target port number
        #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..))]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    //println!("{:?}", cli);
    match &cli.command {
        Some(Commands::Send {
            destination,
            source,
            target,
            port,
        }) => {
            //println!("Send {destination:?},{source:?},{target:?},{port:?}");
            send(destination, source, target, port);
        }
        Some(Commands::Run {
            destination,
            command,
            port,
        }) => {
            //println!("Send {destination:?},{command:?},{port:?}");
            run(destination, command, port);
        }
        _ => (),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
