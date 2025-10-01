use cctmog_protocol::{StoredMessage, MessageScope};
use std::path::Path;
use std::fs;
use std::io::{self, BufRead};
use serde_json;
use tokio::fs as async_fs;
use zeromq::{Socket, SocketSend, ZmqMessage};

pub struct MessageStore {
    data_dir: String,
    zmq_publisher: Option<std::sync::Arc<tokio::sync::Mutex<zeromq::PubSocket>>>,
}

impl MessageStore {
    pub fn new(data_dir: &str) -> io::Result<Self> {
        fs::create_dir_all(data_dir)?;
        Ok(MessageStore {
            data_dir: data_dir.to_string(),
            zmq_publisher: None,
        })
    }

    pub async fn new_with_zmq(data_dir: &str, zmq_port: u16) -> io::Result<Self> {
        fs::create_dir_all(data_dir)?;

        // Initialize ZMQ publisher
        let mut publisher = zeromq::PubSocket::new();
        let zmq_addr = format!("tcp://127.0.0.1:{}", zmq_port);

        if let Err(e) = publisher.bind(&zmq_addr).await {
            eprintln!("Failed to bind ZMQ publisher to {}: {}", zmq_addr, e);
            return Ok(MessageStore {
                data_dir: data_dir.to_string(),
                zmq_publisher: None,
            });
        }

        println!("📡 ZMQ publisher bound to {}", zmq_addr);

        Ok(MessageStore {
            data_dir: data_dir.to_string(),
            zmq_publisher: Some(std::sync::Arc::new(tokio::sync::Mutex::new(publisher))),
        })
    }

    pub async fn store_message(&self, message: &StoredMessage) -> io::Result<()> {
        let file_path = self.get_file_path(&message.scope, &message.room, &message.recipient);

        // Ensure directory exists
        if let Some(parent) = Path::new(&file_path).parent() {
            async_fs::create_dir_all(parent).await?;
        }

        // Append message to file (each message on a new line as JSON)
        let json_line = format!("{}\n", serde_json::to_string(message).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, e)
        })?);

        let content_to_write = if Path::new(&file_path).exists() {
            format!("{}{}", async_fs::read_to_string(&file_path).await.unwrap_or_default(), json_line)
        } else {
            json_line.clone()
        };

        async_fs::write(&file_path, content_to_write).await?;

        // Publish message via ZMQ if publisher is available
        if let Some(ref publisher_mutex) = self.zmq_publisher {
            let topic = format!("chat.{}", match &message.scope {
                MessageScope::Global => "global".to_string(),
                MessageScope::Match => message.room.as_ref().unwrap_or(&"unknown".to_string()).clone(),
                MessageScope::Group => "group".to_string(),
                MessageScope::Private => format!("private.{}", message.recipient.unwrap_or_default()),
            });

            let zmq_message = ZmqMessage::try_from(format!("{} {}", topic, json_line.trim()))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("ZMQ message error: {}", e)))?;

            // Try to send, but don't fail if ZMQ send fails (fallback to file only)
            let mut publisher = publisher_mutex.lock().await;
            if let Err(e) = publisher.send(zmq_message).await {
                eprintln!("Failed to publish message via ZMQ: {}", e);
            }
        }

        Ok(())
    }

    pub async fn get_messages(&self, scope: MessageScope, room: Option<&str>, recipient: Option<uuid::Uuid>, limit: Option<usize>) -> io::Result<Vec<StoredMessage>> {
        let file_path = self.get_file_path(&scope, &room.map(|s| s.to_string()), &recipient);

        if !Path::new(&file_path).exists() {
            return Ok(vec![]);
        }

        let content = async_fs::read_to_string(&file_path).await?;
        let mut messages = Vec::new();

        for line in content.lines() {
            if !line.trim().is_empty() {
                match serde_json::from_str::<StoredMessage>(line) {
                    Ok(message) => messages.push(message),
                    Err(_) => continue, // Skip invalid JSON lines
                }
            }
        }

        // Return most recent messages first, limited by the limit parameter
        messages.reverse();
        if let Some(limit) = limit {
            messages.truncate(limit);
        }

        Ok(messages)
    }

    fn get_file_path(&self, scope: &MessageScope, room: &Option<String>, recipient: &Option<uuid::Uuid>) -> String {
        match scope {
            MessageScope::Match => {
                let default_room = "default".to_string();
                let room_name = room.as_ref().unwrap_or(&default_room);
                format!("{}/match_{}.jsonl", self.data_dir, room_name)
            },
            MessageScope::Group => {
                format!("{}/group.jsonl", self.data_dir)
            },
            MessageScope::Global => {
                format!("{}/global.jsonl", self.data_dir)
            },
            MessageScope::Private => {
                // For private messages, we could store them per recipient or in a shared private folder
                if let Some(recipient_id) = recipient {
                    format!("{}/private_{}.jsonl", self.data_dir, recipient_id)
                } else {
                    format!("{}/private.jsonl", self.data_dir)
                }
            },
        }
    }

    pub async fn clean_old_messages(&self, days_old: u64) -> io::Result<()> {
        // Implementation for cleaning old messages (optional enhancement)
        // For now, we'll keep all messages
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_message_store() {
        let temp_dir = tempdir().unwrap();
        let store = MessageStore::new(temp_dir.path().to_str().unwrap()).unwrap();

        let message = StoredMessage {
            player_name: "TestPlayer".to_string(),
            message: "Hello World!".to_string(),
            scope: MessageScope::Match,
            room: Some("testroom".to_string()),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            recipient: None,
        };

        // Store message
        store.store_message(&message).await.unwrap();

        // Retrieve messages
        let messages = store.get_messages(MessageScope::Match, Some("testroom"), None, None).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].player_name, "TestPlayer");
        assert_eq!(messages[0].message, "Hello World!");
    }
}