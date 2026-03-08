use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc::Sender, Mutex};
use uuid::Uuid;

use crate::{app_state::ManagedState, frontend_commands::ExecSessionRequest};

pub type ExecSessionId = Uuid;

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

impl ManagedState for ExecSessions {
    type WrappedState = Arc<ExecSessions>;

    fn build(_: tauri::AppHandle) -> Self::WrappedState {
        Arc::new(Self::default())
    }
}

impl ExecSessions {
    pub async fn register(&self, sender: Sender<ExecSessionRequest>) -> ExecSessionId {
        let uuid = Uuid::new_v4();

        self.senders.lock().await.insert(uuid, sender);

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
