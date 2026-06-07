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
            let first_byte=msg_buffer[0];
            let opcode=first_byte & 0x0F;
            if opcode == 0x8 {
                println!("client requested connection closure");
                break;
            }
            if opcode==0x1{
                println!("text frame detected");
                let second_byte=msg_buffer[1];
                let mut payload_length=(second_byte & 0x7F) as usize;

                let mask_key=&msg_buffer[2..6];
                let raw_payload=&msg_buffer[6..6+payload_length];

                let mut decoded_payload=Vec::new();
                for i in 0..payload_length{
                    let original_byte=raw_payload[i]^mask_key[i%4];
                    decoded_payload.push(original_byte);
                }
                match String::from_utf8(decoded_payload){
                    Ok(text)=>{
                        println!("decoded message from client {}",text);
                    }
                    Err(e)=>{
                        println!("failed to convert payload to string");
                    }
                }

                let reply="message received";
                let reply_bytes=reply.as_bytes();
                let reply_len=reply_bytes.len();

                let mut frame=Vec::new();
                frame.push(0x81);
                frame.push(reply_len as u8);
                frame.extend_from_slice(reply_bytes);
                socket.write_all(&frame).await.unwrap();


            }

        }
        
    }
}
