#![allow(missing_docs)]

use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::UdpSocket;
use tracing::debug;

use crate::error::Error;
use crate::packet::Packet;

pub async fn create_broadcast_socket() -> Result<UdpSocket, Error> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;
    Ok(socket)
}

pub async fn create_unicast_socket() -> Result<(UdpSocket, SocketAddr), Error> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let local = socket.local_addr()?;
    Ok((socket, local))
}

pub async fn send_to(
    socket: &UdpSocket,
    addr: SocketAddr,
    packet: &Packet,
) -> Result<(), Error> {
    let bytes = packet.to_bytes()?;
    debug!("Sending {} bytes to {addr}", bytes.len());
    socket.send_to(&bytes, addr).await?;
    Ok(())
}

pub async fn broadcast(
    socket: &UdpSocket,
    packet: &Packet,
    port: u16,
) -> Result<(), Error> {
    let addr = SocketAddr::new("255.255.255.255".parse().unwrap(), port);
    send_to(socket, addr, packet).await
}

pub async fn recv_from(
    socket: &UdpSocket,
    timeout: Duration,
    buf: &mut [u8],
) -> Result<(usize, SocketAddr), Error> {
    let result = tokio::time::timeout(timeout, socket.recv_from(buf)).await;

    match result {
        Ok(Ok((size, addr))) => Ok((size, addr)),
        Ok(Err(e)) => Err(Error::Io(e)),
        Err(_) => Err(Error::Timeout),
    }
}

pub async fn send_and_recv(
    socket: &UdpSocket,
    addr: SocketAddr,
    packet: &Packet,
    timeout: Duration,
) -> Result<Packet, Error> {
    send_to(socket, addr, packet).await?;

    let mut buf = vec![0u8; 4096];
    let (size, _) = recv_from(socket, timeout, &mut buf).await?;

    let response = Packet::from_bytes(&buf[..size])?;
    debug!("Received response from {addr}: {:?}", response.t);
    Ok(response)
}

pub async fn send_broadcast_and_collect(
    socket: &UdpSocket,
    packet: &Packet,
    port: u16,
    timeout: Duration,
) -> Result<Vec<Packet>, Error> {
    broadcast(socket, packet, port).await?;

    let mut packets = Vec::new();
    let mut buf = vec![0u8; 4096];

    loop {
        match recv_from(socket, timeout, &mut buf).await {
            Ok((size, addr)) => {
                match Packet::from_bytes(&buf[..size]) {
                    Ok(p) => {
                        debug!("Received packet from {addr}");
                        packets.push(p);
                    }
                    Err(e) => {
                        debug!("Failed to parse packet from {addr}: {e}");
                    }
                }
            }
            Err(Error::Timeout) => break,
            Err(e) => return Err(e),
        }
    }

    Ok(packets)
}

pub fn socket_addr(ip: &str, port: u16) -> SocketAddr {
    SocketAddr::new(ip.parse().unwrap(), port)
}
