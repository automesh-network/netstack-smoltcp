# offload benchmark 

## cargo bench

run `crago [+nightly] bench`, you can get the similar results here below:

```txt
test wire::bench_emit_tcp          ... bench:          55 ns/iter (+/- 5)
test wire::bench_emit_udp          ... bench:          47 ns/iter (+/- 2)
test wire::bench_emit_udp_offload  ... bench:           9 ns/iter (+/- 0)
test wire::bench_parse_tcp         ... bench:          42 ns/iter (+/- 1)
test wire::bench_parse_tcp_offload ... bench:           9 ns/iter (+/- 0)
```

so, we may easily conclude that the bottleneck of ingress/egress of userspace netstack is the checksum of Layer4

## iperf3 throughput bench

### preparation

assume the iperf3 server machine's public ip is a.b.c.d

1. `cargo run --release --example proxy[_lwip] --features [default|offload] -- --interface wlo1 > /tmp/smoltcp.log` (to redirect the output to log file is crucial to the performance)   
2. `ip rule add to a.b.c.d table 200`  
3. `ip route add default dev utun8 table 200`  

then, run `iperf3 -s` on the server side, run `iperf3 --client a.b.c.d -t 60` on the client side

### bench environment

#### machine configs

1. iperf3 client1
    - ISP: Oracle Public Cloud
    - geo location: korea
    - os: Ubuntu 20.04.6 LTS
    - mm: 1G
    - cpu: x86_64; 2 core; AMD EPYC 7551 32-Core Processor
2. iperf3 client2
    - ISP: Oracle Public Cloud
    - geo location: usa
    - os: Ubuntu 22.04.4 LTS
    - mm: 1G
    - cpu: x86_64; 2 core; AMD EPYC 7551 32-Core Processor
3. iperf3 server
    - ISP: Oracle Public Cloud
    - geo location: korea
    - os: Ubuntu 20.04.6 LTS
    - mm: 1G
    - cpu: x86_64; 2 core; AMD EPYC 7551 32-Core Processor

#### iperf3 bench result


**client1 -> server(high workload, the bottleneck is CPU & IO)**

the `htop` shows that, when running our proxy based on tun&smoltcp, the avarage load of 2 cpu cores is about 70%

```txt

[ ID] Interval           Transfer     Bitrate         Retr

# bare connection, without tun
[  5]   0.00-10.00  sec   592 MBytes   497 Mbits/sec    0             sender
[  5]   0.00-10.04  sec   589 MBytes   492 Mbits/sec                  receiver

# with tun of smoltcp, with offload
[  5]   0.00-60.01  sec  1.05 GBytes   151 Mbits/sec   84             sender
[  5]   0.00-60.05  sec  1.05 GBytes   150 Mbits/sec                  receiver

# with tun of smoltcp, without offload
[  5]   0.00-60.00  sec  1.07 GBytes   153 Mbits/sec   83             sender
[  5]   0.00-60.05  sec  1.06 GBytes   152 Mbits/sec                  receiver

# with tun of lwip, without offload(not supported yet)
[  5]   0.00-60.00  sec   536 MBytes  75.0 Mbits/sec   32             sender
[  5]   0.00-60.05  sec   535 MBytes  74.7 Mbits/sec                  receiver

```

**client2 -> server (low workload, the bottleneck is IO)**

the `htop` shows that, when running our proxy based on tun&smoltcp, the avarage load of 2 cpu cores is about 25%

```txt

[ ID] Interval           Transfer     Bitrate         Retr

# bare connection, without tun
[  5]   0.00-60.00  sec   355 MBytes  49.6 Mbits/sec  4435             sender
[  5]   0.00-60.18  sec   353 MBytes  49.2 Mbits/sec                  receiver

# with tun of smoltcp, with offload
[  5]   0.00-60.00  sec   354 MBytes  49.5 Mbits/sec   26             sender
[  5]   0.00-60.20  sec   348 MBytes  48.5 Mbits/sec                  receiver

# with tun of smoltcp, without offload
[  5]   0.00-60.00  sec   341 MBytes  47.7 Mbits/sec   15             sender
[  5]   0.00-60.18  sec   335 MBytes  46.6 Mbits/sec                  receiver

# with tun of lwip, without offload(not supported yet)
[  5]   0.00-60.00  sec   352 MBytes  49.1 Mbits/sec    4             sender
[  5]   0.00-60.18  sec   350 MBytes  48.8 Mbits/sec                  receiver

```

### LAN testing
client - WSL on Intel(R) Core(TM) i9-9900KF CPU @ 3.60GHz w 64G mem
server - DS1821+
networking - Gigabyte cabled

#### Direct

```
➜  ~ iperf3 -c 10.0.0.11 -t 60
Connecting to host 10.0.0.11, port 5201
[  5] local 172.18.165.26 port 53660 connected to 10.0.0.11 port 5201
[ ID] Interval           Transfer     Bitrate         Retr  Cwnd
[  5]   0.00-1.00   sec   112 MBytes   942 Mbits/sec  642   1.55 MBytes
[  5]   1.00-2.00   sec   110 MBytes   923 Mbits/sec    0   1.65 MBytes
[  5]   2.00-3.00   sec   112 MBytes   944 Mbits/sec    0   1.73 MBytes
[  5]   3.00-4.00   sec   111 MBytes   933 Mbits/sec    0   1.79 MBytes
[  5]   4.00-5.00   sec   112 MBytes   944 Mbits/sec    0   1.83 MBytes
[  5]   5.00-6.00   sec   111 MBytes   933 Mbits/sec   54    716 KBytes
[  5]   6.00-7.00   sec   112 MBytes   944 Mbits/sec    0    826 KBytes
[  5]   7.00-8.00   sec   111 MBytes   933 Mbits/sec    0    925 KBytes
[  5]   8.00-9.00   sec   112 MBytes   944 Mbits/sec    0   1011 KBytes
```

#### smoltcp wo/ offload

CPU usage

```
  PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM     TIME+ COMMAND
 9062 root      20   0 1089516   5780   2748 S 152.2   0.0   0:19.82 proxy
 ```

```
➜  ~ iperf3 -c 10.0.0.11 -t 60
Connecting to host 10.0.0.11, port 5201
[  5] local 10.10.10.2 port 51834 connected to 10.0.0.11 port 5201
[ ID] Interval           Transfer     Bitrate         Retr  Cwnd
[  5]   0.00-1.00   sec  99.9 MBytes   838 Mbits/sec    0    676 KBytes
[  5]   1.00-2.00   sec   101 MBytes   849 Mbits/sec    0    676 KBytes
[  5]   2.00-3.00   sec   101 MBytes   849 Mbits/sec    0    676 KBytes
[  5]   3.00-4.00   sec   101 MBytes   849 Mbits/sec    0    676 KBytes
[  5]   4.00-5.00   sec   100 MBytes   839 Mbits/sec    0    676 KBytes
[  5]   5.00-6.00   sec   101 MBytes   849 Mbits/sec    0    676 KBytes
[  5]   6.00-7.00   sec   100 MBytes   839 Mbits/sec    0    676 KBytes
[  5]   7.00-8.00   sec   104 MBytes   870 Mbits/sec    0    676 KBytes
[  5]   8.00-9.00   sec   100 MBytes   839 Mbits/sec    0    676 KBytes
[  5]   9.00-10.00  sec   101 MBytes   849 Mbits/sec    0    676 KBytes
[  5]  10.00-11.00  sec   100 MBytes   839 Mbits/sec    0    676 KBytes
[  5]  11.00-12.00  sec   101 MBytes   849 Mbits/sec    0    676 KBytes
```

#### smoltcp single thread wo/ offload

CPU usage
```
  PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM     TIME+ COMMAND
25773 root      20   0 1089488   7624   3100 S  26.9   0.0   0:09.55 proxy
```

```
➜  ~ iperf3 -c 10.0.0.11 -t 60
Connecting to host 10.0.0.11, port 5201
[  5] local 10.10.10.2 port 41298 connected to 10.0.0.11 port 5201
[ ID] Interval           Transfer     Bitrate         Retr  Cwnd
[  5]   0.00-1.00   sec   116 MBytes   974 Mbits/sec    0    662 KBytes
[  5]   1.00-2.00   sec   111 MBytes   933 Mbits/sec    0    662 KBytes
[  5]   2.00-3.00   sec   112 MBytes   944 Mbits/sec    0    662 KBytes
[  5]   3.00-4.00   sec   112 MBytes   944 Mbits/sec    0    662 KBytes
[  5]   4.00-5.00   sec   111 MBytes   933 Mbits/sec    0    662 KBytes
[  5]   5.00-6.00   sec   111 MBytes   933 Mbits/sec    0    662 KBytes
[  5]   6.00-7.00   sec   110 MBytes   923 Mbits/sec    0    662 KBytes
[  5]   7.00-8.00   sec   111 MBytes   933 Mbits/sec    0    662 KBytes
[  5]   8.00-9.00   sec   112 MBytes   944 Mbits/sec    0    662 KBytes
[  5]   9.00-10.00  sec   111 MBytes   933 Mbits/sec    0    662 KBytes
[  5]  10.00-11.00  sec   112 MBytes   944 Mbits/sec    0    662 KBytes
[  5]  11.00-12.00  sec   111 MBytes   933 Mbits/sec    0    662 KBytes
[  5]  12.00-13.00  sec   112 MBytes   944 Mbits/sec    0    662 KBytes
[  5]  13.00-14.00  sec   111 MBytes   933 Mbits/sec    0    662 KBytes
```

#### smoltcp w offload

CPU usage
```
  PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM     TIME+ COMMAND
 9956 root      20   0 1089516   5788   2996 S 150.8   0.0   0:28.69 proxy
```
```
➜  ~ iperf3 -c 10.0.0.11 -t 60
Connecting to host 10.0.0.11, port 5201
[  5] local 10.10.10.2 port 59138 connected to 10.0.0.11 port 5201
[ ID] Interval           Transfer     Bitrate         Retr  Cwnd
[  5]   0.00-1.00   sec  95.6 MBytes   802 Mbits/sec    0    335 KBytes
[  5]   1.00-2.00   sec  99.1 MBytes   831 Mbits/sec    0    335 KBytes
[  5]   2.00-3.00   sec   103 MBytes   865 Mbits/sec    0    335 KBytes
[  5]   3.00-4.00   sec   106 MBytes   890 Mbits/sec    0    335 KBytes
[  5]   4.00-5.00   sec  99.7 MBytes   836 Mbits/sec    0    335 KBytes
[  5]   5.00-6.00   sec  99.9 MBytes   838 Mbits/sec    0    335 KBytes
[  5]   6.00-7.00   sec   101 MBytes   845 Mbits/sec    0    335 KBytes
[  5]   7.00-8.00   sec  99.7 MBytes   836 Mbits/sec    0    335 KBytes
[  5]   8.00-9.00   sec  98.5 MBytes   827 Mbits/sec    0    335 KBytes
[  5]   9.00-10.00  sec  98.6 MBytes   827 Mbits/sec    0    335 KBytes
[  5]  10.00-11.00  sec  98.7 MBytes   828 Mbits/sec    0    335 KBytes
[  5]  11.00-12.00  sec  94.1 MBytes   789 Mbits/sec    0    335 KBytes
[  5]  12.00-13.00  sec   100 MBytes   841 Mbits/sec    0    335 KBytes
[  5]  13.00-14.00  sec  97.7 MBytes   820 Mbits/sec    0    335 KBytes
```

#### lwip

CPU usage
```
  PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM     TIME+ COMMAND
14153 root      20   0 1090012   5212   2860 S 147.5   0.0   0:26.03 proxy_lwip
```

```
➜  ~ iperf3 -c 10.0.0.11 -t 60
Connecting to host 10.0.0.11, port 5201
[  5] local 10.10.10.2 port 60812 connected to 10.0.0.11 port 5201
[ ID] Interval           Transfer     Bitrate         Retr  Cwnd
[  5]   0.00-1.00   sec  85.4 MBytes   716 Mbits/sec    1   49.9 KBytes
[  5]   1.00-2.00   sec  82.9 MBytes   696 Mbits/sec    0   49.9 KBytes
[  5]   2.00-3.00   sec  83.4 MBytes   699 Mbits/sec    0   49.9 KBytes
[  5]   3.00-4.00   sec  83.9 MBytes   704 Mbits/sec    0   49.9 KBytes
[  5]   4.00-5.00   sec  84.3 MBytes   707 Mbits/sec    0   49.9 KBytes
[  5]   5.00-6.00   sec  83.5 MBytes   700 Mbits/sec    0   49.9 KBytes
[  5]   6.00-7.00   sec  85.3 MBytes   716 Mbits/sec    0   49.9 KBytes
[  5]   7.00-8.00   sec  83.9 MBytes   704 Mbits/sec    0   49.9 KBytes
[  5]   8.00-9.00   sec  84.3 MBytes   707 Mbits/sec    0   49.9 KBytes
[  5]   9.00-10.00  sec  81.5 MBytes   684 Mbits/sec    0   49.9 KBytes
[  5]  10.00-11.00  sec  82.4 MBytes   691 Mbits/sec    0   49.9 KBytes
[  5]  11.00-12.00  sec  84.7 MBytes   710 Mbits/sec    0   49.9 KBytes
```

### flamegraphs

#### Prerequisites

``

#### run

replace the command: `cargo run --release --example proxy --features default -- --interface wlo1 > /tmp/smoltcp.log`  
with the new one: `cargo flamegraph --release --example proxy --features default -- --interface wlo1 > /tmp/smoltcp.log`


#### results

1. client1->server: see [results1](./flamegraphs/client1)
2. client2->server: see [results2](./flamegraphs/client2)

### conslusions

1. `netstack-smoltcp` is better than `netstack-lwip`, it's performances in high CPU workload situations are more desirable; on the platform that both netstack crate is supported, you should always consider `netstack-smoltcp`
2. the checksum offload **CANNOT** lead to a huge leap in iperf throughput, but in the debug version(not listed), the difference is noticable, i guess that's because the debug version of binary file contains more instructions, so the CPU becomes a bottleneck, and the checksum offload of TUN can alleviate this issue
3. the `#[tokio:main]` macro [is a multi threaded runtime](https://docs.rs/tokio-macros/latest/tokio_macros/attr.main.html#using-the-multi-thread-runtime)

### Thanks

- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [iperf](https://github.com/esnet/iperf)