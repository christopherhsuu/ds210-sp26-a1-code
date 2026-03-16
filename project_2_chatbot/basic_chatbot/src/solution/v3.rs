use kalosm::language::*;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct ChatbotV3 {
    // What should you store inside your Chatbot type?
    // The model? The chat_session?
    // Storing a single chat session is not enough: it mixes messages from different users
    // together!
    // Need to store one chat session per user.
    // Think of some kind of data structure that can help you with this.
    model: Llama,
    sessions: HashMap<String, Chat<Llama>>,
}

impl ChatbotV3 {
    #[allow(dead_code)]
    pub fn new(model: Llama) -> ChatbotV3 {
        return ChatbotV3 {
            // Make sure you initialize your struct members here
            model,
            sessions: HashMap::new(),
        };
    }

    #[allow(dead_code)]
    pub async fn chat_with_user(&mut self, username: String, message: String) -> String {
        if !self.sessions.contains_key(&username) {
            let chat_session = self.model
                .chat()
                .with_system_prompt("The assistant will act like a pirate");
            self.sessions.insert(username.clone(), chat_session);
        }
        let session = self.sessions.get_mut(&username).unwrap();
        let mut response_stream = session.add_message(message);
        response_stream.to_std_out().await.unwrap();
        response_stream.all_text().await
    }

    #[allow(dead_code)]
    pub fn get_history(&self, username: String) -> Vec<String> {
        // Extract the chat message history for the given username
        // Hint: think of how you can retrieve the Chat object for that user, when you retrieve it
        // you may want to use https://docs.rs/kalosm/0.4.0/kalosm/language/struct.Chat.html#method.session
        // to then retrieve the history!
        if let Some(chat) = self.sessions.get(&username) {
        chat.session()
            .history() 
            .into_iter()
            .map(|msg| msg.content().to_string())
            .collect()
    } else {
        Vec::new()
    }
    }
}