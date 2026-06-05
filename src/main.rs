mod state;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt,AsyncWriteExt};

use state::{User,Room,ChatAppRoom};

#[tokio::main]
async fn main() ->Result<(),Box<dyn std::error::Error>>{
    let listener=TcpListener::bind("127.0.0.1:8080").await?;
    println!("server is live and listening to 127.0.0.1:8080");
    loop{
        let (socket,addr)=listener.accept().await?;
        println!("new connection established from {}",addr);
        tokio::spawn(async move {
            handle_connection(socket).await;

        });
    }
}

async fn handle_connection(mut socket:tokio::net::TcpStream){
    let mut buffer=[0;1024];
    let bytes_read=socket.read(&mut buffer).await.unwrap();
    let request=String::from_utf8_lossy(&buffer[..bytes_read]);
    // println!("raw request {}",request);
    let mut handshake_success=false;
    for line in request.lines(){
        if line.starts_with("Sec-WebSocket-Key:"){
            let key=line.split(":").nth(1).unwrap().trim();
            // println!("isolated key {}",key);
            let magic_guid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
            let combined=format!("{}{}",key,magic_guid);

            use sha1::{Sha1,Digest};
            let mut hasher=Sha1::new();
            hasher.update(combined.as_bytes());
            let hash_res=hasher.finalize();

            use base64::{prelude::BASE64_STANDARD,Engine};
            let accept_key=BASE64_STANDARD.encode(hash_res);
            let response = format!(
"HTTP/1.1 101 Switching Protocols\r\n\
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: {}\r\n\r\n",
                accept_key
            );
            socket.write_all(response.as_bytes()).await.unwrap();
            handshake_success=true;
            break;
        }
    }
    if handshake_success{
        println!("handshake successfull. keep connection alive...");
        loop{
            let mut msg_buffer=[0;1024];
            let n=socket.read(&mut msg_buffer).await.unwrap();
            if n==0{
                println!("client disconnected");
                break;
            }
            println!("received {} raw binary data bytes from upgraded stream",n);

        }
        
    }
}
