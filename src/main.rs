mod state;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt,AsyncWriteExt};

use state::{User,Room,ChatAppRoom};
use std::sync::{Arc,Mutex};




#[tokio::main]
async fn main() ->Result<(),Box<dyn std::error::Error>>{
    let shared_state=Arc::new(Mutex::new(ChatAppRoom::new()));
   
    
    let listener=TcpListener::bind("127.0.0.1:8080").await?;
    println!("server is live and listening to 127.0.0.1:8080");
    loop{
        let (socket,addr)=listener.accept().await?;
        println!("new connection established from {}",addr);
        let state_clone=Arc::clone(&shared_state);
        tokio::spawn(async move {
            handle_connection(socket,state_clone).await;

        });
    }
}

async fn handle_connection(mut socket:tokio::net::TcpStream,state:Arc<Mutex<ChatAppRoom>>){
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
        
        let mut join_buffer=[0;1024];
        let n=socket.read(&mut join_buffer).await.unwrap();
        let second_byte=join_buffer[1];
        let len=(second_byte & 0x7F) as usize;
        let mask=&join_buffer[2..6];
        let payload=&join_buffer[6..6+len];

        let mut decoded=Vec::new();
        for i in 0..len{
            decoded.push(payload[i]^mask[i%4]);
        }
        let join_text=String::from_utf8(decoded).unwrap();
        let mut room_tx = None;
        let mut rx = None;
        let mut client_username = String::new();
        if join_text.starts_with("JOIN:"){
            let parts:Vec<&str>=join_text.split(':').collect();
            let room_name=parts[1].to_string();
            client_username=parts[2].to_string();
            let mut state_guard=state.lock().unwrap();
            let room=state_guard.chatroom.entry(room_name.clone()).or_insert_with(||{
                let (tx,_)=tokio::sync::broadcast::channel(10);
                Room{
                    room_name,
                    tx,
                    members:Vec::new()
                }
            });
            rx = Some(room.tx.subscribe());
            room_tx = Some(room.tx.clone());
            room.members.push(User{
                user_name:client_username.clone()
            });
            
            println!("successfully assigned client : {} to group",client_username);
        }
        let room_tx = room_tx.unwrap();
        let mut rx = rx.unwrap();
        loop {
            let mut msg_buffer = [0; 1024];
            
            tokio::select! {
                // EVENT A: Read incoming data from the browser socket
                read_result = socket.read(&mut msg_buffer) => {
                    let n = read_result.unwrap();
                    if n == 0 {
                        println!("client disconnected");
                        break;
                    }
                    
                    let first_byte = msg_buffer[0];
                    let opcode = first_byte & 0x0F;
                    
                    if opcode == 0x8 {
                        println!("client requested connection closure");
                        break;
                    }
                    
                    if opcode == 0x1 {
                        let second_byte = msg_buffer[1];
                        let payload_length = (second_byte & 0x7F) as usize;

                        let mask_key = &msg_buffer[2..6];
                        let raw_payload = &msg_buffer[6..6+payload_length];

                        let mut decoded_payload = Vec::new();
                        for i in 0..payload_length {
                            let original_byte = raw_payload[i] ^ mask_key[i % 4];
                            decoded_payload.push(original_byte);
                        }
                        
                        if let Ok(text) = String::from_utf8(decoded_payload) {
                            println!("decoded message from client {}: {}", client_username, text);
                            
                            // Format the message nicely and broadcast it to the entire room tower!
                            let broadcast_msg = format!("{}: {}", client_username, text);
                            let _ = room_tx.send(broadcast_msg);
                        }
                    }
                }
                
                // EVENT B: Listen for incoming broadcasts from other clients in this room
                broadcast_result = rx.recv() => {
                    if let Ok(msg) = broadcast_result {
                        let reply_bytes = msg.as_bytes();
                        let reply_len = reply_bytes.len();

                        // Package the broadcasted text into an unmasked frame
                        let mut frame = Vec::new();
                        frame.push(0x81);
                        frame.push(reply_len as u8);
                        frame.extend_from_slice(reply_bytes);
                        
                        // Blast it down to our client's browser!
                        socket.write_all(&frame).await.unwrap();
                    }
                }
            }
        }
    
        
    }
}
