use std::net::IpAddr;

pub fn send(destination: &IpAddr, source: &str, target: &str, port: &u16) {
    println!("{},{},{},{}", destination, source, target, port);
}

pub fn run(destination: &IpAddr, command: &str, port: &u16) {
    println!("{},{},{}", destination, command, port);
}
