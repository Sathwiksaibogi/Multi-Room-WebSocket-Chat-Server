use std::collections::HashMap;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct User{
    pub user_name:String,

}
pub struct Room{
    pub room_name:String,
    pub tx:broadcast::Sender<String>,
    pub members:Vec<User>,

}
pub struct ChatAppRoom{
    pub chatroom:HashMap<String,Room>,

}

impl ChatAppRoom{
    pub fn new()-> Self{
        ChatAppRoom { 
            chatroom:HashMap::new(),
         }
    }
}