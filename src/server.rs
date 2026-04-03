use std::collections::HashMap;
use tokio::{sync::mpsc};
use crate::{messages::{UserRole, WsMessage}};
pub struct SessionInfo {
    pub tx : mpsc::UnboundedSender<WsMessage>,
    pub role: UserRole
}

pub struct WashServer {
    pub sessions: HashMap<usize, SessionInfo>,
}

impl WashServer {
    pub fn new() ->Self {
        Self { sessions: (HashMap::new()) }
    }

    pub fn connexion(&mut self, id: usize, tx: mpsc::UnboundedSender<WsMessage>, role: UserRole) {
        let session= SessionInfo {
            tx,
            role,
        };

        self.sessions.insert(id, session);
        
    }
    
    pub fn deconnexion(&mut self, id: usize) {
        
        self.sessions.remove(&id);
    } 
    pub fn broadcast(&self, msg:WsMessage) {
        for session in self.sessions.values() {
            let _ = session.tx.send(msg.clone());

        }
        
    }
}