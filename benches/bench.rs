#![feature(test)]

mod wire {
    use smoltcp::phy::Checksum;
    use smoltcp::phy::ChecksumCapabilities;
    use smoltcp::wire::{IpAddress, Ipv4Address};
    use smoltcp::wire::{TcpControl, TcpPacket, TcpRepr, TcpSeqNumber};
    use smoltcp::wire::{UdpPacket, UdpRepr};

    extern crate test;

    const SRC_ADDR: IpAddress = IpAddress::Ipv4(Ipv4Address([192, 168, 1, 1]));
    const DST_ADDR: IpAddress = IpAddress::Ipv4(Ipv4Address([192, 168, 1, 2]));

    // cannot be offloaded
    #[bench]
    fn bench_emit_tcp(b: &mut test::Bencher) {
        use smoltcp::phy::Checksum;

        static PAYLOAD_BYTES: [u8; 400] = [0x2a; 400];
        let repr: TcpRepr<'static> = TcpRepr {
            src_port: 48896,
            dst_port: 80,
            control: TcpControl::Syn,
            seq_number: TcpSeqNumber(0x01234567),
            ack_number: None,
            window_len: 0x0123,
            window_scale: None,
            max_seg_size: None,
            sack_permitted: false,
            sack_ranges: [None, None, None],
            payload: &PAYLOAD_BYTES,
        };
        let mut bytes = vec![0xa5; repr.buffer_len()];
        let mut cap = ChecksumCapabilities::default();
        cap.tcp = Checksum::Tx;

        b.iter(|| {
            let mut packet = TcpPacket::new_unchecked(&mut bytes);
            repr.emit(&mut packet, &SRC_ADDR, &DST_ADDR, &cap);
        });
    }

    fn construct_tcp_packet() -> Vec<u8> {
        static PAYLOAD_BYTES: [u8; 400] = [0x2a; 400];
        let repr: TcpRepr<'static> = TcpRepr {
            src_port: 48896,
            dst_port: 80,
            control: TcpControl::Syn,
            seq_number: TcpSeqNumber(0x01234567),
            ack_number: None,
            window_len: 0x0123,
            window_scale: None,
            max_seg_size: None,
            sack_permitted: false,
            sack_ranges: [None, None, None],
            payload: &PAYLOAD_BYTES,
        };
        let mut bytes = vec![0xa5; repr.buffer_len()];
        let cap = ChecksumCapabilities::default();

        let mut packet = TcpPacket::new_unchecked(&mut bytes);
        repr.emit(&mut packet, &SRC_ADDR, &DST_ADDR, &cap);
        bytes
    }

    #[bench]
    fn bench_parse_tcp(b: &mut test::Bencher) {
        let bytes = construct_tcp_packet();
        let cap = ChecksumCapabilities::default();

        b.iter(|| {
            let packet = TcpPacket::new_unchecked(&bytes[..]);
            let _ = TcpRepr::parse(&packet, &SRC_ADDR, &DST_ADDR, &cap);
        });
    }

    #[bench]
    fn bench_parse_tcp_offload(b: &mut test::Bencher) {
        let bytes = construct_tcp_packet();
        let mut cap = ChecksumCapabilities::default();
        cap.tcp = Checksum::Tx;

        b.iter(|| {
            let packet = TcpPacket::new_unchecked(&bytes[..]);
            let _ = TcpRepr::parse(&packet, &SRC_ADDR, &DST_ADDR, &cap);
        });
    }

    #[bench]
    fn bench_emit_udp(b: &mut test::Bencher) {
        static PAYLOAD_BYTES: [u8; 400] = [0x2a; 400];
        let repr = UdpRepr {
            src_port: 48896,
            dst_port: 80,
        };
        let mut bytes = vec![0xa5; repr.header_len() + PAYLOAD_BYTES.len()];
        let cap = ChecksumCapabilities::default();

        b.iter(|| {
            let mut packet = UdpPacket::new_unchecked(&mut bytes);
            repr.emit(
                &mut packet,
                &SRC_ADDR,
                &DST_ADDR,
                PAYLOAD_BYTES.len(),
                |buf| buf.copy_from_slice(&PAYLOAD_BYTES),
                &cap,
            );
        });
    }

    #[bench]
    fn bench_emit_udp_offload(b: &mut test::Bencher) {
        static PAYLOAD_BYTES: [u8; 400] = [0x2a; 400];
        let repr = UdpRepr {
            src_port: 48896,
            dst_port: 80,
        };
        let mut bytes = vec![0xa5; repr.header_len() + PAYLOAD_BYTES.len()];
        let mut cap = ChecksumCapabilities::default();
        cap.udp = Checksum::None;

        b.iter(|| {
            let mut packet = UdpPacket::new_unchecked(&mut bytes);
            repr.emit(
                &mut packet,
                &SRC_ADDR,
                &DST_ADDR,
                PAYLOAD_BYTES.len(),
                |buf| buf.copy_from_slice(&PAYLOAD_BYTES),
                &cap,
            );
        });
    }
}
