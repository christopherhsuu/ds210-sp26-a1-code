use kalosm::language::*;
use file_chatbot::solution::file_library;

use crate::solution::Cache;

pub struct ChatbotV5 {
    model: Llama,
    cache: Cache<Chat<Llama>>,
}

impl ChatbotV5 {
    pub fn new(model: Llama) -> ChatbotV5 {
        return ChatbotV5 {
            model: model,
            cache: Cache::new(3),
        };
    }

    pub async fn chat_with_user(&mut self, username: String, message: String) -> String {
        let filename = format!("{}.txt", username);
        let cached_chat = self.cache.get_chat(&username);

        match cached_chat {
            None => {
                println!("chat_with_user: {username} is not in the cache!");
                let mut chat = self.model
                    .chat()
                    .with_system_prompt("The assistant will act like a pirate");
                if let Some(session) = file_library::load_chat_session_from_file(&filename) {
                    chat = chat.with_session(session);
                }
                self.cache.insert_chat(username.clone(), chat);
                let chat = self.cache.get_chat(&username).unwrap();
                let response = chat(&message).await.unwrap();
                let session = chat.session().unwrap();
                file_library::save_chat_session_to_file(&filename, &session);
                response.to_string()
            }
            Some(chat_session) => {
                println!("chat_with_user: {username} is in the cache! Nice!");
                let response = chat_session(&message).await.unwrap();
                let session = chat_session.session().unwrap();
                file_library::save_chat_session_to_file(&filename, &session);
                response.to_string()
            }
        }
    }

    pub fn get_history(&mut self, username: String) -> Vec<String> {
        let filename = &format!("{}.txt", username);
        let cached_chat: Option<&mut Chat<Llama>> = self.cache.get_chat(&username);

        match cached_chat {
            None => {
                let mut chat: Chat<Llama> = self.model
                    .chat()
                    .with_system_prompt("The assistant will act like a pirate");
                if let Some(session) = file_library::load_chat_session_from_file(filename) {
                    chat = chat.with_session(session);
                }

                let history: Vec<String> = chat.session().unwrap().history()
                    .iter()
                    .skip(1)
                    .map(|msg: &ChatMessage| msg.content().to_string())
                    .collect();

                self.cache.insert_chat(username, chat);
                
                history
            }

            Some(chat_session) => {
                println!("get_history: {username} is in the cache! Nice!");
                let history = chat_session.session().unwrap().history();
                history.iter().skip(1).map(|msg: &ChatMessage| msg.content().to_string()).collect()
            }
        }
    }
}