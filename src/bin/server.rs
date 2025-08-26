use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader}
};

use serde::{Deserialize, Serialize};

use chrono::Local;

use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage{
    username: String,
    content: String,
    timestamp: String,
    message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    UserMessage,
    SystemNotification,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let listener = TcpListener::bind("127.0.0.1:8082").await?;

    println!("Chat server started on {}", listener.local_addr()?);
    println!("press Ctrl+C to stop the server");

    let (tx, _) = broadcast::channel::<String>(100);

    loop {
        let (socket, addr) = listener.accept().await?;

        println!("[{}] New Connection", Local::now().format("%d-%m-%Y %H:%M:%S"));
        println!("Address: {}", addr);

        let tx = tx.clone();
        let rx = tx.subscribe();

        // tokio::spawn(async move{
        //     handle_connection()
        // })
    }

    async fn handle_connection(
        mut socket: TcpStream,
        tx: broadcast::Sender<String>,
        mut rx: broadcast::Receiver<String>,
    ){
        let (reader, mut writer) = socket.split();
        let mut reader = BufReader::new(reader);
        let mut username = String::new();

        reader.read_line(&mut username).await.unwrap();
        let username = username.trim().to_string();

        let join_msg = ChatMessage{
            username: username.clone(),
            content: "joined the chat".to_string(),
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            message_type: MessageType::SystemNotification,
        };

        let join_json = serde_json::to_string(&join_msg).unwrap();

        tx.send(join_json).unwrap();
    }

}
