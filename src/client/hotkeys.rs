use super::Client;
use crate::{
    requests::hotkeys::{KeyModifiers, Request},
    responses::hotkeys as responses,
    Result,
};

/// API functions related to hotkeys.
pub struct Hotkeys<'a> {
    pub(super) client: &'a Client,
}

impl<'a> Hotkeys<'a> {
    /// Gets an array of all hotkey names in OBS.
    pub async fn list(&self) -> Result<Vec<String>> {
        self.client
            .send_message::<_, responses::Hotkeys>(Request::List)
            .await
            .map(|h| h.hotkeys)
    }

    /// Triggers a hotkey using its name. See [`Self::list`].
    pub async fn trigger_by_name(&self, name: &str) -> Result<()> {
        self.client
            .send_message(Request::TriggerByName { name })
            .await
    }

    /// Triggers a hotkey using a sequence of keys.
    pub async fn trigger_by_sequence(&self, id: &str, modifiers: KeyModifiers) -> Result<()> {
        self.client
            .send_message(Request::TriggerBySequence { id, modifiers })
            .await
    }
}
