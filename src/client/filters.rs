use serde::{de::DeserializeOwned, Serialize};

use super::Client;
use crate::{
    requests::filters::{
        Create, CreateInternal, Request, SetEnabled, SetIndex, SetName, SetSettings,
        SetSettingsInternal,
    },
    responses::filters as responses,
    Error, Result,
};

/// API functions related to filters.
pub struct Filters<'a> {
    pub(super) client: &'a Client,
}

impl<'a> Filters<'a> {
    /// Gets an array of all of a source's filters.
    pub async fn list(&self, source: &str) -> Result<Vec<responses::SourceFilter>> {
        self.client
            .send_message::<_, responses::Filters>(Request::List { source })
            .await
            .map(|f| f.filters)
    }

    /// Gets the default settings for a filter kind.
    pub async fn default_settings<T>(&self, kind: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.client
            .send_message::<_, responses::DefaultFilterSettings<T>>(Request::DefaultSettings {
                kind,
            })
            .await
            .map(|dfs| dfs.default_filter_settings)
    }

    /// Creates a new filter, adding it to the specified source.
    pub async fn create<T>(&self, filter: Create<'_, T>) -> Result<()>
    where
        T: Serialize,
    {
        self.client
            .send_message(Request::Create(CreateInternal {
                source: filter.source,
                filter: filter.filter,
                kind: filter.kind,
                settings: filter
                    .settings
                    .map(|settings| serde_json::to_value(&settings))
                    .transpose()
                    .map_err(Error::SerializeCustomData)?,
            }))
            .await
    }

    /// Removes a filter from a source.
    pub async fn remove(&self, source: &str, filter: &str) -> Result<()> {
        self.client
            .send_message(Request::Remove { source, filter })
            .await
    }

    /// Sets the name of a source filter (rename).
    pub async fn set_name(&self, name: SetName<'_>) -> Result<()> {
        self.client.send_message(Request::SetName(name)).await
    }

    /// Gets the info for a specific source filter.
    pub async fn get(&self, source: &str, filter: &str) -> Result<responses::SourceFilter> {
        self.client
            .send_message(Request::Get { source, filter })
            .await
    }

    /// Sets the index position of a filter on a source.
    pub async fn set_index(&self, index: SetIndex<'_>) -> Result<()> {
        self.client.send_message(Request::SetIndex(index)).await
    }

    /// Sets the settings of a source filter.
    pub async fn set_settings<T>(&self, settings: SetSettings<'_, T>) -> Result<()>
    where
        T: Serialize,
    {
        self.client
            .send_message(Request::SetSettings(SetSettingsInternal {
                source: settings.source,
                filter: settings.filter,
                settings: serde_json::to_value(&settings.settings)
                    .map_err(Error::SerializeCustomData)?,
                overlay: settings.overlay,
            }))
            .await
    }

    /// Sets the enable state of a source filter.
    pub async fn set_enabled(&self, enabled: SetEnabled<'_>) -> Result<()> {
        self.client.send_message(Request::SetEnabled(enabled)).await
    }
}
