use std::collections::HashMap;
use tokio::{sync::mpsc};
use crate::messages::{UserRole, WsMessage};

/// Información de una sesión WebSocket conectada.
pub struct SessionInfo {
    pub tx: mpsc::UnboundedSender<WsMessage>,
    pub role: UserRole,
}

/// Servidor central que gestiona todas las sesiones conectadas.
pub struct WashServer {
    pub sessions: HashMap<usize, SessionInfo>,
}

impl WashServer {
    pub fn new() -> Self {
        Self { sessions: HashMap::new() }
    }

    /// Registra una nueva conexión WebSocket.
    pub fn connexion(&mut self, id: usize, tx: mpsc::UnboundedSender<WsMessage>, role: UserRole) {
        let session = SessionInfo { tx, role };
        self.sessions.insert(id, session);
    }

    /// Elimina una sesión al desconectarse.
    pub fn deconnexion(&mut self, id: usize) {
        self.sessions.remove(&id);
    }

    /// Envía un mensaje solo a las sesiones de un **rol específico**.
    pub fn broadcast_to_role(&self, msg: WsMessage, role: UserRole) {
        for session in self.sessions.values() {
            if session.role == role {
                let _ = session.tx.send(msg.clone());
            }
        }
    }

    /// Envía un mensaje a **varios roles** a la vez (ej: Supervisor + Cajero).
    pub fn broadcast_to_roles(&self, msg: WsMessage, roles: &[UserRole]) {
        for session in self.sessions.values() {
            if roles.contains(&session.role) {
                let _ = session.tx.send(msg.clone());
            }
        }
    }
}