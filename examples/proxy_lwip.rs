use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};
use netstack_lwip::{NetStack, TcpListener, UdpSocket};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::warn;
use tun::{Device, TunPacket};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(short = "i", long = "interface")]
    interface: String,
}

// to run this example, you should set the policy routing **after the start of the main program**
// the cmds can be:
// 1. `cargo run --example proxy --features default -- --interface wlo1`
// 2. `ip rule add to 1.1.1.1 table 200`
// 3. `ip route add default dev utun8 table 200`
// 4. `curl 1.1.1.1` or or run the netperf(https://github.com/ahmedsoliman/netperf) test of tcp stream
// currently, the example only supports the TCP stream, and the UDP packet will be dropped.

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

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
    let (stack, tcp_listener, udp_socket) = NetStack::new().unwrap();

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
        handle_inbound_stream(tcp_listener, &opt.interface).await;
    });

    // Receive and send UDP packets between netstack and NAT manager. The NAT
    // manager would maintain UDP sessions and send them to the dispatcher.
    let f4 = tokio::spawn(async move {
        handle_inbound_datagram(udp_socket).await;
    });

    let res = futures::future::join_all(vec![f1, f2, f3, f4]).await;
    for r in res {
        if let Err(e) = r {
            tracing::error!("error: {:?}", e);
        }
    }
}

// simply forward
async fn handle_inbound_stream(mut tcp_listener: TcpListener, interface: &str) {
    loop {
        while let Some((mut stream, local_addr, remote_addr)) = tcp_listener.next().await {
            let interface = interface.to_owned();
            tokio::spawn(async move {
                println!("new tcp connection: {:?} => {:?}", local_addr, remote_addr);
                if let Ok(mut remote) = new_tcp_stream(remote_addr, &interface).await {
                    match tokio::io::copy_bidirectional(&mut stream, &mut remote).await {
                        Ok(_) => {}
                        Err(e) => warn!(
                            "failed to copy tcp stream {:?}=>{:?}, err: {:?}",
                            local_addr, remote_addr, e
                        ),
                    }
                }
            });
        }
    }
}
async fn handle_inbound_datagram(mut udp_socket: Box<UdpSocket>) {
    loop {
        let n = udp_socket.next().await;
        let (_data, src, dst) = match n {
            Some((data, src, dst)) => (data, src, dst),
            None => {
                warn!("failed to get udp packet");
                continue;
            }
        };

        tracing::warn!("dropping packet from {:?} to {:?}", src, dst);
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

pub trait ProxyStream: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T> ProxyStream for T where T: AsyncRead + AsyncWrite + Send + Unpin {}
pub type AnyStream = Box<dyn ProxyStream>;

pub async fn new_tcp_stream(addr: SocketAddr, iface: &str) -> std::io::Result<AnyStream> {
    use shadowsocks::net::*;
    let mut opts: ConnectOpts = ConnectOpts::default();
    opts.bind_interface = Some(iface.to_owned());

    shadowsocks::net::TcpStream::connect_with_opts(&addr, &opts)
        .await
        .map(|x| Box::new(x) as _)
}

pub async fn new_udp_packet(
    addr: &SocketAddr,
    iface: &str,
) -> std::io::Result<tokio::net::UdpSocket> {
    use shadowsocks::net::*;
    let mut opts = ConnectOpts::default();
    opts.bind_interface = Some(iface.to_owned());

    shadowsocks::net::UdpSocket::connect_with_opts(addr, &opts)
        .await
        .map(|x| x.into())
}
