mod state;

use tokio::net::TcpListener;

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

async fn handle_connection(socket:tokio::net::TcpStream){
    println!("communicating with client");
}
