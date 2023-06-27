#[allow(unused)]
use clap::Parser;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::net::IpAddr;
use std::net::TcpStream;
use std::thread;
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
        /// Target node IP:PORT addresses to send the file to
        #[arg(short, long)]
        ip_ports: Vec<String>,
        /// Source file to send
        #[arg(short, long)]
        source: String,
        /// Target file path
        #[arg(short, long)]
        target: String,
    },
    Run {
        /// Target node IP address to send the file to
        #[arg(short, long, value_parser = clap::value_parser!(IpAddr))]
        ip_ports: Vec<String>,
        /// Remote command to run
        #[arg(short, long)]
        command: String,
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
            ip_ports,
            source,
            target,
        }) => {
            // Read the file into a byte vector

            let mut file_content = Vec::new();
            let mut file = File::open(source).expect("Error opening file");
            let file_content_size = file
                .read_to_end(&mut file_content)
                .expect("Error reading file");

            let message = Message {
                data: file_content,
                command: Command::Copy {
                    file_path: String::from(target),
                },
            };

            let message_bytes = bincode::serialize(&message).unwrap();
            let message_len = (message_bytes.len() as u32).to_be_bytes();

            send_to_destinations(ip_ports, message_len, message_bytes);
        }
        Some(Commands::Run { ip_ports, command }) => {
            println!("Send {ip_ports:?},{command:?}");
        }
        _ => (),
    }
}

fn send_to_destinations(ip_ports: &Vec<String>, message_len: [u8; 4], message_bytes: Vec<u8>) {
    for destination in ip_ports {
        thread::spawn(move || {
            let mut stream = TcpStream::connect(destination).expect("Failed to connect");

            stream
                .write_all(&message_len)
                .expect("Failed to send message length");

            stream
                .write_all(&message_bytes)
                .expect("Failed to send message");

            stream.flush().expect("Failed to flush stream");

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
        })
        .join()
        .unwrap();
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
