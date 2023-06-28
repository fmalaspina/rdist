#[allow(unused)]
use clap::Parser;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream};
use std::ops::RangeInclusive;
use std::thread;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Send file to target
    Copy {
        /// Target node IP:PORT addresses to send the file to
        #[arg(short, long, value_name = "IP:PORT", required = true,value_parser = valid_ip_port)]
        destinations: Vec<String>,
        /// Source file to send
        #[arg(short, long, required = true)]
        source: String,
        /// Target file path
        #[arg(short, long, required = true)]
        target: String,
    },
    Run {
        /// Target node IP address to send the file to
        #[clap(short, long, value_name = "IP:PORT", required = true,value_parser = valid_ip_port)]
        destinations: Vec<String>,
        /// Remote command to run
        #[clap(short, long, required = true)]
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
    match cli.command {
        Some(Commands::Copy {
            destinations: ip_ports,
            source,
            target,
        }) => {
            // Read the file into a byte vector
            let mut file_content = Vec::new();
            let mut file = File::open(&source).expect("Error opening file");
            file.read_to_end(&mut file_content)
                .expect("Error reading file");

            let message = Message {
                data: file_content,
                command: Command::Copy {
                    file_path: String::from(&target),
                },
            };

            let message_bytes = bincode::serialize(&message).unwrap();
            let message_len = (message_bytes.len() as u32).to_be_bytes();
            for destination in ip_ports.into_iter() {
                let message_bytes_cloned = message_bytes.clone();
                let message_len_cloned = message_len.clone();
                send_command(destination, message_bytes_cloned, message_len_cloned);
            }
        }

        Some(Commands::Run {
            destinations: ip_ports,
            command,
        }) => {
            let message = Message {
                data: vec![],
                command: Command::Run {
                    command: String::from(&command),
                },
            };

            let message_bytes = bincode::serialize(&message).unwrap();
            let message_len = (message_bytes.len() as u32).to_be_bytes();
            for destination in ip_ports.into_iter() {
                let message_bytes_cloned = message_bytes.clone();
                let message_len_cloned = message_len.clone();
                send_command(destination, message_bytes_cloned, message_len_cloned);
            }
        }
        None => {
            println!(
                "No arguments or commands passed. Nothing to do. Use -h or --help for help. Bye!"
            )
        }
    }
}

fn send_command(destination: String, message_bytes: Vec<u8>, message_len: [u8; 4]) {
    thread::spawn(move || {
        let mut stream = TcpStream::connect(&destination).expect("Failed to connect");

        stream
            .write(&message_len)
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
        println!("Response from {}:\n{}", destination, response_string);

        stream
            .shutdown(std::net::Shutdown::Write)
            .expect("Failed to shutdown stream");
    })
    .join()
    .unwrap();
    // }
}

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

fn valid_ip_port(s: &str) -> Result<String, String> {
    let ip_port: Vec<&str> = s.split(":").collect();
    let ip = ip_port[0];
    ip.parse::<IpAddr>()
        .map_err(|_| format!("`{ip}` isn't a valid IP address"))?;

    let port = ip_port[1];
    port.parse::<u16>()
        .map_err(|_| format!("`{port}` isn't a valid port number"))?;
    if PORT_RANGE.contains(&port.parse::<usize>().unwrap()) {
        Ok(s.to_string())
    } else {
        Err(format!(
            "port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
