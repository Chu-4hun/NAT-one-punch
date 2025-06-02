use std::net::Ipv4Addr;

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// Client name
    #[arg(long, short, default_value = "one")]
    pub name: String,
    /// Peer name to connect to
    #[arg(long, short, default_value = "other")]
    pub peer: String,
    /// Rendezvous server IP (should be a public IP)
    #[arg(long, default_value = "45.151.30.139")]
    pub rendezvous: Ipv4Addr,

    /// http port for  Rendezvous server
    #[arg(long, default_value = "8080")]
    pub rendezvous_http_port: u16,

    /// udp port for  Rendezvous server
    #[arg(long, default_value = "4200")]
    pub rendezvous_udp_port: u16,
}
