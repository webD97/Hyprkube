use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc::Sender, Mutex};
use uuid::Uuid;

use crate::frontend_commands::ExecSessionRequest;

pub type ExecSessionId = Uuid;
pub type ExecSessionsState = Arc<ExecSessions>;

#[derive(thiserror::Error, Debug)]
pub enum ExecSessionError {
    #[error(transparent)]
    AsyncSendError(#[from] tokio::sync::mpsc::error::SendError<ExecSessionRequest>),
    #[error("Session with ID {0} not found")]
    SessionNotFound(ExecSessionId),
}

impl serde::Serialize for ExecSessionError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Default)]
pub struct ExecSessions {
    senders: Mutex<HashMap<Uuid, Sender<ExecSessionRequest>>>,
}

impl ExecSessions {
    pub fn new_state() -> ExecSessionsState {
        Arc::new(Self::default())
    }

    pub async fn register(&self, sender: Sender<ExecSessionRequest>) -> ExecSessionId {
        let uuid = Uuid::new_v4();

        self.senders.lock().await.insert(uuid.clone(), sender);

        uuid
    }

    pub async fn send(
        &self,
        console_id: &ExecSessionId,
        request: ExecSessionRequest,
    ) -> Result<(), ExecSessionError> {
        let senders = self.senders.lock().await;
        let sender = senders
            .get(console_id)
            .ok_or(ExecSessionError::SessionNotFound(*console_id))?;

        Ok(sender.send(request).await?)
    }
}
