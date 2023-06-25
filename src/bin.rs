#[allow(unused)]
use clap::Parser;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs::File;

use std::io::{Read, Write};
use std::net::IpAddr;
use std::net::TcpStream;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Send file to target
    Copy {
        /// Config file
        #[arg(short, long)]
        config: String,
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
        /// Config file
        #[arg(short, long)]
        config: String,
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
#[derive(Serialize, Deserialize)]
struct Message {
    data: Vec<u8>,
    command: Command,
}
#[derive(Serialize, Deserialize)]
enum Command {
    Copy { file_path: String },
    Run { command: String },
}

fn main() {
    let cli = Cli::parse();

    //println!("{:?}", cli);
    match &cli.command {
        Some(Commands::Copy {
            destination,
            source,
            target,
            port,
        }) => {
            //println!("Send {destination:?},{source:?},{target:?},{port:?}");
            let mut stream =
                TcpStream::connect(format!("{}:{}", destination, port)).expect("Failed to connect");

            // Read the file into a byte vector
            let mut file_content = Vec::new();
            let mut file = File::open(source).expect("Error opening file");
            let file_content_size = file
                .read_to_end(&mut file_content)
                .expect("Error reading file");

            // Create the message struct
            let message = Message {
                data: file_content,
                command: Command::Copy {
                    file_path: String::from(target),
                },
            };

            // Serialize the message to bytes
            let message_bytes = bincode::serialize(&message).unwrap();

            // Send message length
            let message_len = (message_bytes.len() as u32).to_be_bytes();

            stream
                .write_all(&message_len)
                .expect("Failed to send message length");

            // Send the message
            stream
                .write_all(&message_bytes)
                .expect("Failed to send message");
            stream.flush().expect("Failed to flush stream");
            println!(
                "Message length sent: {}, file contents length sent: {}",
                message_bytes.len(),
                file_content_size
            );
            // Read the response from the server if needed
            let mut response_len_buffer = [0u8; 4];
            stream
                .read_exact(&mut response_len_buffer)
                .expect("Failed to read response length");

            let response_len = u32::from_be_bytes(response_len_buffer);
            let mut response_buffer = vec![0u8; response_len as usize];
            stream
                .read_exact(&mut response_buffer)
                .expect("Failed to read response data");
            // Convert the response buffer to a string
            let response_string = String::from_utf8_lossy(&response_buffer);

            // Print the response
            println!("Response: {}", response_string);
            stream
                .shutdown(std::net::Shutdown::Write)
                .expect("Failed to shutdown stream");
        }
        Some(Commands::Run {
            destination,
            command,
            port,
        }) => {
            println!("Send {destination:?},{command:?},{port:?}");
        }
        _ => (),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
