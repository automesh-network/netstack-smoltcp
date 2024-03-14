use futures::{SinkExt, StreamExt};
use netstack_smoltcp::{self, StackBuilder, TcpListener, UdpSocket};
use tun::{Device, TunPacket};

fn main() {
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
    tokio::spawn(async move {
        while let Some(pkt) = stack_stream.next().await {
            if let Ok(pkt) = pkt {
                tun_sink.send(TunPacket::new(pkt)).await.unwrap();
            }
        }
    });

    // Reads packet from TUN and sends to stack.
    tokio::spawn(async move {
        while let Some(pkt) = tun_stream.next().await {
            if let Ok(pkt) = pkt {
                stack_sink.send(pkt.into_bytes().into()).await.unwrap();
            }
        }
    });

    // Extracts TCP connections from stack and sends them to the dispatcher.
    tokio::spawn(async move {
        handle_inbound_stream(tcp_listener).await;
    });

    // Receive and send UDP packets between netstack and NAT manager. The NAT
    // manager would maintain UDP sessions and send them to the dispatcher.
    tokio::spawn(async move {
        handle_inbound_datagram(udp_socket).await;
    });
}

async fn handle_inbound_stream(_tcp_listener: TcpListener) {
    /* TODO */
}

async fn handle_inbound_datagram(_udp_socket: UdpSocket) {
    /* TODO */
}

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
