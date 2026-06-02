use std::collections::HashMap;


pub struct User{
    pub user_name:String,

}
pub struct Room{
    pub room_name:String,
    pub members:Vec<User>,

}
pub struct ChatAppRoom{
    pub chatroom:HashMap<String,Room>,

}