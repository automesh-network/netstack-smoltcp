use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};
use netstack_smoltcp::{self, StackBuilder, TcpListener, UdpSocket};
use tokio::net::{TcpSocket, TcpStream};
use tracing::{debug, warn};
use tun::{Device, TunPacket};


// to run this example, you should set the policy routing **after the start of the main program**
// the rules can be:
// `ip rule add to 1.1.1.1 table 200`
// `ip route add default dev utun8 table 200`

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .finish(),
    )
    .unwrap();

    let mut cfg = tun::Configuration::default();
    cfg.layer(tun::Layer::L3);
    #[cfg(target_os = "linux")]
    cfg.platform(|cfg| {
        // Provides pure IP packet
        cfg.packet_information(false);
    });
    let fd = -1;
    if fd >= 0 {
        cfg.raw_fd(fd);
    } else {
        cfg.name("utun8")
            .address("10.10.10.2")
            .destination("10.10.10.1")
            .mtu(1504);
        #[cfg(not(any(
            target_arch = "mips",
            target_arch = "mips64",
            target_arch = "mipsel",
            target_arch = "mipsel64",
        )))]
        {
            cfg.netmask("255.255.255.0");
        }
        cfg.up();
    }

    let device = tun::create_as_async(&cfg).unwrap();
    let mut builder = StackBuilder::default();
    if let Some(device_broadcast) = get_device_broadcast(&device) {
        builder = builder
            .add_src_v4_filter(move |v4| *v4 == device_broadcast)
            .add_dst_v4_filter(move |v4| *v4 == device_broadcast);
    }
    let (udp_socket, tcp_listener, stack) = builder.run();

    let framed = device.into_framed();
    let (mut tun_sink, mut tun_stream) = framed.split();
    let (mut stack_sink, mut stack_stream) = stack.split();

    // Reads packet from stack and sends to TUN.
    let f1 = tokio::spawn(async move {
        while let Some(pkt) = stack_stream.next().await {
            if let Ok(pkt) = pkt {
                match tun_sink.send(TunPacket::new(pkt)).await {
                    Ok(_) => {}
                    Err(e) => warn!("failed to send packet to TUN, err: {:?}", e),
                }
            }
        }
    });

    // Reads packet from TUN and sends to stack.
    let f2 = tokio::spawn(async move {
        while let Some(pkt) = tun_stream.next().await {
            if let Ok(pkt) = pkt {
                match stack_sink.send(pkt.into_bytes().into()).await {
                    Ok(_) => {}
                    Err(e) => warn!("failed to send packet to stack, err: {:?}", e),
                };
            }
        }
    });

    // Extracts TCP connections from stack and sends them to the dispatcher.
    let f3 = tokio::spawn(async move {
        handle_inbound_stream(tcp_listener).await;
    });

    // Receive and send UDP packets between netstack and NAT manager. The NAT
    // manager would maintain UDP sessions and send them to the dispatcher.
    let f4 = tokio::spawn(async move {
        handle_inbound_datagram(udp_socket).await;
    });

    let res = futures::future::join_all(vec![f1, f2, f3, f4]).await;
    for r in res {
        if let Err(e) = r {
            eprintln!("error: {:?}", e);
        }
    }
}

// simply forward
async fn handle_inbound_stream(mut tcp_listener: TcpListener) {
    loop {
        while let Some((mut stream, local, remote)) = tcp_listener.next().await {
            println!("new tcp connection: {:?} => {:?}", local, remote);
            let mut remote = new_tcp_stream(remote, "wlo1").await.unwrap();
            match tokio::io::copy_bidirectional(&mut stream, &mut remote).await {
                Ok(_) => {}
                Err(e) => warn!(
                    "failed to copy tcp stream {:?}=>{:?}, err: {:?}",
                    local, remote, e
                ),
            }
        }
    }
    /* TODO */
}

async fn handle_inbound_datagram(mut udp_socket: UdpSocket) {
    let mut recv_buf = vec![0; 1024 * 64];
    loop {
        while let Some((data, src, dst)) = udp_socket.next().await {
            if dst != "1.1.1.1:53".parse().unwrap() {
                continue;
            }
            println!("new udp packet: {:?} => {:?}", src, dst);
            let interface = Some("wlo1");
            let outbound = new_udp_packet(interface).await.unwrap();
            match outbound.send_to(&data, dst).await {
                Ok(_) => {}
                Err(e) => {
                    warn!(
                        "failed to send udp packet to outbound, src: {:?}, dst: {:?}, err:{:?}",
                        src, dst, e
                    );
                    continue;
                }
            };
            let res = outbound.recv(&mut recv_buf).await.unwrap();
            debug!("recv {:?} <= {:?}, packet size:{}", src, dst, res);
            match udp_socket.send((recv_buf[..res].to_vec(), dst, src)).await {
                Ok(_) => {}
                Err(e) => {
                    warn!(
                        "failed to send udp packet to stack, src: {:?}, dst: {:?}, err:{:?}",
                        src, dst, e
                    );
                    continue;
                }
            }
        }
    }
    /* TODO */
}

#[allow(unused)]
fn get_device_broadcast(device: &tun::AsyncDevice) -> Option<std::net::Ipv4Addr> {
    let mtu = device.get_ref().mtu().unwrap_or(1500);

    let address = match device.get_ref().address() {
        Ok(a) => a,
        Err(err) => {
            println!("[alrd] failed to get tun device address, error: {}", err);
            return None;
        }
    };

    let netmask = match device.get_ref().netmask() {
        Ok(n) => n,
        Err(err) => {
            println!("[alrd] failed to get tun device netmask, error: {}", err);
            return None;
        }
    };

    match smoltcp::wire::Ipv4Cidr::from_netmask(address.into(), netmask.into()) {
        Ok(address_net) => match address_net.broadcast() {
            Some(broadcast) => {
                println!(
                    "[alrd] tun device network: {} (address: {}, netmask: {}, mtu: {})",
                    address_net, address, netmask, mtu,
                );

                Some(broadcast.into())
            }
            None => {
                println!(
                    "[alrd] invalid tun address {}, netmask {}",
                    address, netmask
                );
                None
            }
        },
        Err(err) => {
            println!(
                "[alrd] invalid tun address {}, netmask {}, error: {}",
                address, netmask, err
            );
            None
        }
    }
}

pub async fn new_tcp_stream<'a>(addr: SocketAddr, iface: &str) -> std::io::Result<TcpStream> {
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)?;

    socket.bind_device(Some(iface.as_bytes()))?;
    socket.set_keepalive(true)?;
    socket.set_nodelay(true)?;
    socket.set_nonblocking(true)?;

    let stream = TcpSocket::from_std_stream(socket.into())
        .connect(addr)
        .await?;

    Ok(stream)
}

pub async fn new_udp_packet(iface: Option<&str>) -> std::io::Result<tokio::net::UdpSocket> {
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?;
    if let Some(iface) = iface {
        socket.bind_device(Some(iface.as_bytes()))?;
    }
    socket.set_nonblocking(true)?;

    tokio::net::UdpSocket::from_std(socket.into())
}
