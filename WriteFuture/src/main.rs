use std::{
    future::Future,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    pin::Pin,
    task::{Context, Poll},
    thread::{self, JoinHandle},
};

use tokio::runtime::Runtime;

struct WriteFuture<'a> {
    socket: TcpStream,
    data: &'a [u8],
}

impl<'a> WriteFuture<'a> {
    fn new(socket: TcpStream, data: &'a [u8]) -> Self {
        socket.set_nonblocking(true).unwrap();
        Self { socket, data }
    }
}

impl Future for WriteFuture<'_> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let data = self.data;

        match self.socket.write(data) {
            Ok(length) => Poll::Ready(Ok(length)),
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

fn main() {
    let server = run_server();
    let client = run_client();

    server.join().unwrap();
    client.join().unwrap();
}

const ADDR: &'static str = "127.0.0.1:18373";

fn run_server() -> JoinHandle<()> {
    thread::spawn(|| {
        let listener = TcpListener::bind(ADDR).unwrap();

        let (mut client_accepted, _addr) = listener.accept().unwrap();

        let mut message = String::new();
        client_accepted.read_to_string(&mut message).unwrap();
        dbg!(message);
    })
}

fn run_client() -> JoinHandle<()> {
    thread::spawn(|| {
        let client = TcpStream::connect(ADDR).unwrap();
        
        let mut rt = Runtime::new().unwrap();
        rt.block_on(WriteFuture::new(client, b"Hello, world!")).unwrap();
    })
}