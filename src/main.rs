use std::{net::SocketAddrV4, str::FromStr, time::Duration};

use anyhow::Context;
use clap::Parser;
use reqwest::StatusCode;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UdpSocket,
    time::sleep,
};
use tracing::{error, info, warn};

use crate::args::Cli;

mod args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    // Регистрируемся и получаем UDPSocket для использования в будущем
    let socket = register(
        &SocketAddrV4::new(cli.rendezvous, cli.rendezvous_udp_port),
        &cli.name,
    )
    .await?;
    let http_soc_addr = SocketAddrV4::new(cli.rendezvous, cli.rendezvous_http_port);

    let mut peer: Option<SocketAddrV4> = None;
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();
    let mut recv_buf = [0; 1024];

    loop {
        if let Some(peer_addr) = peer {
            tokio::select! {
                res = reader.read_line(&mut input) => {
                    match res {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            info!("Sending message: {}", input.trim_end());
                            if let Err(e) = socket.send_to(input.as_bytes(), peer_addr).await {
                                error!("Failed to send message: {}", e);
                            }
                            input.clear();
                        }
                        Err(e) => {
                            error!("Stdin read error: {}", e);
                            break;
                        }
                    }
                }
                res = socket.recv_from(&mut recv_buf) => {
                    match res {
                        Ok((len, addr)) => {
                            if addr == std::net::SocketAddr::V4(peer_addr) {
                                info!("Received: {}", std::str::from_utf8(recv_buf[..len].trim_ascii_end())?);
                            } else {
                                warn!("Ignoring packet from unexpected source: {}", addr);
                            }
                        }
                        Err(e) => error!("Socket receive error: {}", e),
                    }
                }
            }
        } else {
            // this doesn't work properly all the time because of weird caching on server 
            match get_peer_address(&http_soc_addr, &cli.peer).await {
                Ok(Some(addr)) => {
                    info!("Peer found at {}", addr);
                    // Send initial hole punching packet
                    if let Err(e) = socket.send_to(&[0], addr).await {
                        error!("Initial punch failed: {}", e);
                    } else {
                        peer = Some(addr);
                    }
                }
                Ok(None) => sleep(Duration::from_secs(1)).await,
                Err(e) => error!("Peer lookup failed: {}", e),
            }
        }
    }

    Ok(())
}

async fn get_peer_address(
    rendezvous: &SocketAddrV4,
    peer: &str,
) -> anyhow::Result<Option<SocketAddrV4>> {
    let url = format!("http://{}/api/wait/{}", rendezvous, peer);
    info!("Querying peer address: {}", url);

    let response = reqwest::get(&url).await.context("HTTP request failed")?;
    info!("HTTP status: {}", response.status());

    match response.status() {
        StatusCode::OK => {
            let text = response.text().await.context("Failed to read response")?;
            SocketAddrV4::from_str(&text)
                .map(Some)
                .map_err(|e| anyhow::anyhow!("Invalid address format: {}", e))
        }
        StatusCode::NOT_FOUND => {
            warn!("Peer '{}' not registered", peer);
            Ok(None)
        }
        status => {
            let body = response.text().await.unwrap_or_default();
            error!("Unexpected HTTP status {}: {}", status, body);
            Err(anyhow::anyhow!("HTTP error {}", status))
        }
    }
}
async fn register(rendezvous: &SocketAddrV4, name: &str) -> anyhow::Result<UdpSocket> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0")
        .await
        .context("register: UdpSocket::bind")?;

    let mut packet = Vec::<u8>::with_capacity(name.len() + 2);

    // Заголовок: 0x00
    packet.push(0x00);

    // Имя клиента: UTF-8 байты строки
    packet.extend(name.as_bytes());

    // Конец пакета: 0xFF
    packet.push(0xFF);

    info!(?rendezvous, "packet {packet:X?}");
    socket
        .send_to(&packet, rendezvous)
        .await
        .context("register: socket.send_to")?;

    Ok(socket)
}
