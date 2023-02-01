use std::{env, io::Cursor};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use async_trait::async_trait;
use protocol::{
    AsyncReadFromBuffer, AsyncWireReadExt, AsyncWireWriteExt, AsyncWriteToBuffer, ExpectedPacketId,
    PacketId, ProtocolError,
};
use tokio::{
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use crate::protocol::{AsyncReadRawPacket, AsyncWriteRawPacket};

pub(crate) mod protocol;
pub mod services;

#[derive(Debug)]
struct HandshakePacket {
    protocol_version: usize,
    server_address: String,
    port: u16,
    next_state: usize,
}

impl ExpectedPacketId for HandshakePacket {
    fn get_expected_packet_id() -> usize {
        0
    }
}

#[async_trait]
impl AsyncReadFromBuffer for HandshakePacket {
    async fn read_from_buffer(buffer: Vec<u8>) -> Result<Self, ProtocolError> {
        let mut reader = Cursor::new(buffer);

        let protocol_version = reader.read_varint().await?;
        let server_address = reader.read_string().await?;
        let port = reader.read_u16().await?;
        let next_state = reader.read_varint().await?;

        Ok(HandshakePacket {
            protocol_version,
            server_address,
            port,
            next_state,
        })
    }
}

#[derive(Debug)]
struct DisconnectPacket {
    packet_id: usize,
    reason: String,
}

impl PacketId for DisconnectPacket {
    fn get_packet_id(&self) -> usize {
        self.packet_id
    }
}

#[async_trait]
impl AsyncWriteToBuffer for DisconnectPacket {
    async fn write_to_buffer(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buffer = Cursor::new(Vec::<u8>::new());

        buffer.write_string(&self.reason).await?;

        Ok(buffer.into_inner())
    }
}

#[async_trait]
pub trait AsyncOldWireWriteExt {
    async fn write_string16(&mut self, s: &str) -> Result<(), ProtocolError>;
}

#[async_trait]
impl<W: AsyncWrite + Unpin + Send + Sync> AsyncOldWireWriteExt for W {
    async fn write_string16(&mut self, s: &str) -> Result<(), ProtocolError> {
        let data = s.encode_utf16().collect::<Vec<u16>>();
        self.write_u16(data.len() as u16).await?;
        for x in data {
            self.write_u16(x).await?;
        }

        Ok(())
    }
}

async fn process(mut socket: TcpStream, _sender: Sender<()>) -> bool {
    let mut buf = bytes::BytesMut::with_capacity(1024);

    // In a loop, read data from the socket and write the data back.
    loop {
        let size = socket.read_buf(&mut buf).await.expect("Cannot read socket");

        let mut cursor = Cursor::new(&buf);

        if buf[0] == 0x01 {
            println!("Login");
            // Old login packet
            let _protocol_version = cursor.read_u32().await.expect("Cannot read packet");
            socket.write_u8(0xFF).await.expect("Cannot write data");
            socket
                .write_string16("You triggered a server start!")
                .await
                .expect("Cannot write data");
        } else if buf[0] == 0x02 {
            println!("Handshake");
        } else {
            // New login packet
            if let Ok(packet) = cursor.read_packet::<HandshakePacket>().await {

                if packet.next_state == 2 {
                    println!("{:?}", packet);
                    // Next state: Login
                    socket
                        .write_packet(DisconnectPacket {
                            packet_id: 0,
                            reason: "\"You triggered a server start!\"".to_string(),
                        })
                        .await
                        .expect("Cannot write packet");
                    break;
                }
            }
        }
    }

    true
}

#[cfg(target_os = "windows")]
fn launch_script() -> std::io::Result<Child> {
    let path = "start.bat";

    let child = Command::new("cmd.exe")
        .current_dir("run")
        .arg("/c")
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    child
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (send, mut recv) = tokio::sync::mpsc::channel(1);

    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:25565 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25565".to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.
        let send_clone = send.clone();
        let result = tokio::spawn(async move {
            process(socket, send_clone).await
        });

        if result.await.unwrap() {
            // Wait for the tasks to finish.
            //
            // We drop our sender first because the recv() call otherwise
            // sleeps forever.
            drop(send);

            // When every sender has gone out of scope, the recv call
            // will return with an error. We ignore the error.
            let _ = recv.recv().await;

            break;
        }
    }

    drop(listener);

    launch_script().expect("Cannot launch script");

    Ok(())
}
