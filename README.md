SYNOPSIS
------

    [shell]# http-queue-lite <ip> <port>
    HTTP Queue Lite Started.
    [shell]# curl http://127.0.0.1:4321/get
    queue empty
    [shell]# curl http://127.0.0.1:4321/add?task1
    added ok
    [shell]# curl http://127.0.0.1:4321/add?task2
    added ok
    [shell]# curl http://127.0.0.1:4321/get
    task1
    [shell]# curl http://127.0.0.1:4321/get
    task2


INSTALL
------

    curl -s http://static.rust-lang.org/rustup.sh | sh
    wget https://github.com/chengang/rust-http-queue-lite/archive/v1.1.tar.gz
    tar xvf v1.1.tar.gz
    cd rust-http-queue-lite-1.1/
    cargo build --release
    install target/release/http-queue-lite /usr/sbin/

USEAGE
------

    http-queue-lite <listen_ip> <listen_port>

    example:
    setsid http-queue-lite 127.0.0.1 4321 &

PERFORMANCE
------

    at Intel(R) Xeon(R) CPU E5-2630 v2 @ 2.60GHz & 32G MEM

    siege -b -c 300 -r 300 "http://127.0.0.1:4321/add?task1"
    Transactions:          90000 hits
    Availability:         100.00 %
    Elapsed time:          24.22 secs
    Data transferred:         0.86 MB
    Response time:            0.08 secs
    Transaction rate:      3715.94 trans/sec
    Throughput:           0.04 MB/sec
    Concurrency:          298.80
    Successful transactions:       90000
    Failed transactions:             0
    Longest transaction:          0.12
    Shortest transaction:         0.00

    siege -b -c 300 -r 300 "http://127.0.0.1:4321/get"
    Transactions:          90000 hits
    Availability:         100.00 %
    Elapsed time:          14.31 secs
    Data transferred:         0.60 MB
    Response time:            0.05 secs
    Transaction rate:      6289.31 trans/sec
    Throughput:           0.04 MB/sec
    Concurrency:          298.68
    Successful transactions:       90000
    Failed transactions:             0
    Longest transaction:          0.06
    Shortest transaction:         0.00
