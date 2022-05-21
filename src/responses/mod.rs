//! All responses that can be received from the API.

pub use semver::Version as SemVerVersion;
use serde::{de, Deserialize, Deserializer};
use serde_repr::Deserialize_repr;
use time::Duration;

use crate::common::{Alignment, BoundsType, MonitorType};

#[derive(Debug)]
pub(crate) enum ServerMessage {
    /// First message sent from the server immediately on client connection. Contains authentication
    /// information if authentication is required. Also contains RPC version for version
    /// negotiation.
    Hello(Hello),
    /// The identify request was received and validated, and the connection is now ready for normal
    /// operation.
    Identified(Identified),
    /// An event coming from OBS has occurred. For example scene switched, source muted.
    #[cfg(feature = "events")]
    Event(crate::events::Event),
    #[cfg(not(feature = "events"))]
    Event,
    /// `obs-websocket` is responding to a request coming from a client.
    RequestResponse(RequestResponse),
    /// `obs-websocket` is responding to a request batch coming from the client.
    RequestBatchResponse(RequestBatchResponse),
}

impl<'de> Deserialize<'de> for ServerMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawServerMessage {
            #[serde(rename = "op")]
            op_code: OpCode,
            #[serde(rename = "d")]
            data: serde_json::Value,
        }

        #[derive(Deserialize_repr)]
        #[repr(u8)]
        enum OpCode {
            /// The initial message sent by obs-websocket to newly connected clients.
            Hello = 0,
            /// The response sent by obs-websocket to a client after it has successfully identified
            /// with obs-websocket.
            Identified = 2,
            /// The message sent by obs-websocket containing an event payload.
            Event = 5,
            /// The message sent by obs-websocket in response to a particular request from a client.
            RequestResponse = 7,
            /// The message sent by obs-websocket in response to a particular batch of requests from
            /// a client.
            RequestBatchResponse = 9,
        }

        let raw = RawServerMessage::deserialize(deserializer)?;

        Ok(match raw.op_code {
            OpCode::Hello => {
                ServerMessage::Hello(serde_json::from_value(raw.data).map_err(de::Error::custom)?)
            }
            OpCode::Identified => ServerMessage::Identified(
                serde_json::from_value(raw.data).map_err(de::Error::custom)?,
            ),
            OpCode::Event => {
                #[cfg(feature = "events")]
                {
                    ServerMessage::Event(
                        serde_json::from_value(raw.data).map_err(de::Error::custom)?,
                    )
                }
                #[cfg(not(feature = "events"))]
                {
                    ServerMessage::Event
                }
            }
            OpCode::RequestResponse => ServerMessage::RequestResponse(
                serde_json::from_value(raw.data).map_err(de::Error::custom)?,
            ),
            OpCode::RequestBatchResponse => ServerMessage::RequestBatchResponse(
                serde_json::from_value(raw.data).map_err(de::Error::custom)?,
            ),
        })
    }
}

/// First message sent from the server immediately on client connection. Contains authentication
/// information if authentication is required. Also contains RPC version for version negotiation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Hello {
    pub obs_web_socket_version: SemVerVersion,
    /// version number which gets incremented on each **breaking change** to the obs-websocket
    /// protocol. Its usage in this context is to provide the current RPC version that the server
    /// would like to use.
    pub rpc_version: u32,
    pub authentication: Option<Authentication>,
}

/// The identify request was received and validated, and the connection is now ready for normal
/// operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Identified {
    /// The RPC (remote procedure call) version to be used.
    pub negotiated_rpc_version: u32,
}

/// `obs-websocket` is responding to a request coming from a client.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RequestResponse {
    pub request_type: String,
    pub request_id: String,
    pub request_status: Status,
    #[serde(default)]
    pub response_data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RequestBatchResponse {
    pub request_id: String,
    pub results: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Authentication {
    pub challenge: String,
    pub salt: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Status {
    /// Is true if the request resulted in [`StatusCode::Success`]. False if otherwise.
    pub result: bool,
    pub code: StatusCode,
    /// May be provided by the server on errors to offer further details on why a request failed.
    pub comment: Option<String>,
}

/// The status code gives information about the result of a request. It gives further insight into
/// what went wrong, if a request failed.
#[derive(Debug, Deserialize_repr)]
#[repr(u16)]
pub enum StatusCode {
    /// Unknown status, should never be used.
    Unknown = 0,

    /// For internal use to signify a successful field check.
    NoError = 10,

    /// The request has succeeded.
    Success = 100,

    /// The `requestType` field is missing from the request data.
    MissingRequestType = 203,
    /// The request type is invalid or does not exist.
    UnknownRequestType = 204,
    /// Generic error code.
    ///
    /// **Note:** A comment is required to be provided by obs-websocket.
    GenericError = 205,
    /// The request batch execution type is not supported.
    UnsupportedRequestBatchExecutionType = 206,

    /// A required request field is missing.
    MissingRequestField = 300,
    /// The request does not have a valid `requestData` object.
    MissingRequestData = 301,

    /// Generic invalid request field message.
    ///
    /// **Note:** A comment is required to be provided by obs-websocket.
    InvalidRequestField = 400,
    /// A request field has the wrong data type.
    InvalidRequestFieldType = 401,
    /// A request field (number) is outside the allowed range.
    RequestFieldOutOfRange = 402,
    /// A request field (string or array) is empty and cannot be.
    RequestFieldEmpty = 403,
    /// There are too many request fields (For example a request takes two optional fields, where
    /// only one is allowed at a time).
    TooManyRequestFields = 404,

    /// An output is running and cannot be in order to perform the request.
    OutputRunning = 500,
    /// An output is not running and should be.
    OutputNotRunning = 501,
    /// An output is paused and should not be.
    OutputPaused = 502,
    /// An output is not paused and should be.
    OutputNotPaused = 503,
    /// An output is disabled and should not be.
    OutputDisabled = 504,
    /// Studio mode is active and cannot be.
    StudioModeActive = 505,
    /// Studio mode is not active and should be.
    StudioModeNotActive = 506,

    /// The resource was not found.
    ///
    /// **Note:** Resources are any kind of object in obs-websocket, like inputs, profiles, outputs,
    /// etc.
    ResourceNotFound = 600,
    /// The resource already exists.
    ResourceAlreadyExists = 601,
    /// The type of resource found is invalid.
    InvalidResourceType = 602,
    /// There are not enough instances of the resource in order to perform the request.
    NotEnoughResources = 603,
    /// The state of the resource is invalid. For example, if the resource is blocked from being
    /// accessed.
    InvalidResourceState = 604,
    /// The specified input (obs_source_t-OBS_SOURCE_TYPE_INPUT) had the wrong kind.
    InvalidInputKind = 605,
    /// The resource does not support being configured.
    ///
    /// This is particularly relevant to transitions, where they do not always have changeable
    /// settings.
    ResourceNotConfigurable = 606,
    /// The specified filter had the wrong kind.
    InvalidFilterKind = 607,

    /// Creating the resource failed.
    ResourceCreationFailed = 700,
    /// Performing an action on the resource failed.
    ResourceActionFailed = 701,
    /// Processing the request failed unexpectedly.
    ///
    /// **Note:** A comment is required to be provided by obs-websocket.
    RequestProcessingFailed = 702,
    /// The combination of request fields cannot be used to perform an action.
    CannotAct = 703,
}

/// Response value for
/// [`get_scene_collection_list`](crate::client::Config::get_scene_collection_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneCollections {
    /// The name of the current scene collection.
    pub current_scene_collection_name: String,
    /// Array of all available scene collections.
    pub scene_collections: Vec<String>,
}

/// Response value for [`get_profile_list`](crate::client::Config::get_profile_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profiles {
    /// The name of the current profile.
    pub current_profile_name: String,
    /// Array of all available profiles.
    pub profiles: Vec<String>,
}

/// Response value for [`get_profile_parameter`](crate::client::Config::get_profile_parameter).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileParameter {
    /// Value associated with the parameter.
    pub parameter_value: Option<String>,
    /// Default value associated with the parameter.
    pub default_parameter_value: Option<String>,
}

/// Response value for [`get_video_settings`](crate::client::Config::get_video_settings).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSettings {
    /// Numerator of the fractional FPS value.
    pub fps_numerator: u32,
    /// Denominator of the fractional FPS value.
    pub fps_denominator: u32,
    /// Width of the base (canvas) resolution in pixels.
    pub base_width: u32,
    /// Height of the base (canvas) resolution in pixels.
    pub base_height: u32,
    /// Width of the output resolution in pixels.
    pub output_width: u32,
    /// Height of the output resolution in pixels.
    pub output_height: u32,
}

/// Response value for
/// [`get_stream_service_settings`](crate::client::Config::get_stream_service_settings).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamServiceSettings<T> {
    /// Stream service type, like `rtmp_custom` or `rtmp_common`.
    pub stream_service_type: String,
    /// Stream service settings.
    pub stream_service_settings: T,
}

/// Response value for [`get_source_filter_list`](crate::client::Filters::get_source_filter_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Filters {
    /// Array of filters.
    pub filters: Vec<SourceFilter>,
}

/// Response value for [`get_source_filter_list`](crate::client::Filters::get_source_filter_list)
/// and [`get_source_filter`](crate::client::Filters::get_source_filter).
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFilter {
    /// Whether the filter is enabled.
    pub filter_enabled: bool,
    /// Index of the filter in the list, beginning at 0.
    pub filter_index: u32,
    /// The kind of filter.
    pub filter_kind: String,
    /// name of the filter.
    #[serde(default)]
    pub filter_name: String,
    /// Settings object associated with the filter.
    pub filter_settings: serde_json::Value,
}

/// Response value for
/// [`get_source_filter_default_settings`](crate::client::Filters::get_source_filter_default_settings).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DefaultFilterSettings<T> {
    /// Object of default settings for the filter kind.
    pub default_filter_settings: T,
}

/// Response value for [`get_version`](crate::client::General::get_version).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    /// Current OBS Studio version.
    pub obs_version: SemVerVersion,
    /// Current obs-websocket version.
    pub obs_web_socket_version: SemVerVersion,
    /// Current latest obs-websocket RPC version.
    pub rpc_version: u32,
    /// Array of available RPC requests for the currently negotiated RPC version.
    pub available_requests: Vec<String>,
    /// Image formats available in `GetSourceScreenshot` and `SaveSourceScreenshot` requests.
    pub supported_image_formats: Vec<String>,
    /// Name of the platform. Usually `windows`, `macos`, or `ubuntu` (Linux flavor). Not guaranteed
    /// to be any of those.
    pub platform: String,
    /// Description of the platform, like `Windows 10 (10.0)`.
    pub platform_description: String,
}

/// Response value for [`get_stats`](crate::client::General::get_stats).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    /// Current CPU usage in percent.
    pub cpu_usage: f64,
    /// Amount of memory in MB currently being used by OBS.
    pub memory_usage: f64,
    /// Available disk space on the device being used for recording storage.
    pub available_disk_space: f64,
    /// Current FPS being rendered.
    pub active_fps: f64,
    /// Average time in milliseconds that OBS is taking to render a frame.
    pub average_frame_render_time: f64,
    /// Number of frames skipped by OBS in the render thread.
    pub render_skipped_frames: u32,
    /// Total number of frames outputted by the render thread.
    pub render_total_frames: u32,
    /// Number of frames skipped by OBS in the output thread.
    pub output_skipped_frames: u32,
    /// Total number of frames outputted by the output thread.
    pub output_total_frames: u32,
    /// Total number of messages received by obs-websocket from the client.
    pub web_socket_session_incoming_messages: u64,
    /// Total number of messages sent by obs-websocket to the client.
    pub web_socket_session_outgoing_messages: u64,
}

/// Response value for [`get_hotkey_list`](crate::client::General::get_hotkey_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Hotkeys {
    /// Array of hotkey names.
    pub hotkeys: Vec<String>,
}

/// Response value for [`get_studio_mode_enabled`](crate::client::Ui::get_studio_mode_enabled).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StudioModeEnabled {
    /// Whether studio mode is enabled.
    pub studio_mode_enabled: bool,
}

/// Response value for [`call_vendor_request`](crate::client::General::call_vendor_request).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CallVendorResponse<T> {
    /// Object containing appropriate response data.
    pub response_data: T,
}

/// Response value for [`get_input_list`](crate::client::Inputs::get_input_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Inputs {
    /// Array of inputs.
    pub inputs: Vec<Input>,
}

/// Response value for [`get_input_list`](crate::client::Inputs::get_input_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    /// Name of the input source.
    pub input_name: String,
    /// Version input kind.
    pub input_kind: String,
    /// Kind of input, without the version part.
    pub unversioned_input_kind: String,
}

/// Response value for [`get_input_kind_list`](crate::client::Inputs::get_input_kind_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InputKinds {
    /// Array of input kinds.
    pub input_kinds: Vec<String>,
}

/// Response value for
/// [`get_input_default_settings`](crate::client::Inputs::get_input_default_settings).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DefaultInputSettings<T> {
    /// Object of default settings for the input kind.
    pub default_input_settings: T,
}

/// Response value for [`get_input_settings`](crate::client::Inputs::get_input_settings).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSettings<T> {
    /// Object of settings for the input.
    pub input_settings: T,
    /// The kind of the input.
    pub input_kind: String,
}

/// Response value for [`get_input_mute`](crate::client::Inputs::get_input_mute) and
/// [`toggle_input_mute`](crate::client::Inputs::toggle_input_mute).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InputMuted {
    /// Whether the input is muted.
    pub input_muted: bool,
}

/// Response value for [`get_input_volume`](crate::client::Inputs::get_input_volume).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputVolume {
    /// Volume setting in mul.
    pub input_volume_mul: f32,
    /// Volume setting in dB.
    pub input_volume_db: f32,
}

/// Response value for
/// [`get_media_input_status`](crate::client::MediaInputs::get_media_input_status).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaStatus {
    /// State of the media input.
    pub media_state: MediaState,
    /// Total duration of the playing media. [`None`] if not playing.
    #[serde(deserialize_with = "crate::de::duration_millis_opt")]
    pub media_duration: Option<Duration>,
    /// Position of the cursor. [`None`] if not playing.
    #[serde(deserialize_with = "crate::de::duration_millis_opt")]
    pub media_cursor: Option<Duration>,
}

/// Response value for
/// [`get_media_input_status`](crate::client::MediaInputs::get_media_input_status) as part of
/// [`MediaStatus`].
#[derive(Copy, Clone, Debug, Deserialize)]
pub enum MediaState {
    /// No state.
    #[serde(rename = "OBS_MEDIA_STATE_NONE")]
    None,
    /// Media is playing.
    #[serde(rename = "OBS_MEDIA_STATE_PLAYING")]
    Playing,
    /// Opening file for replay.
    #[serde(rename = "OBS_MEDIA_STATE_OPENING")]
    Opening,
    /// Buffering data for replay.
    #[serde(rename = "OBS_MEDIA_STATE_BUFFERING")]
    Buffering,
    /// Media is paused.
    #[serde(rename = "OBS_MEDIA_STATE_PAUSED")]
    Paused,
    /// Media stopped.
    #[serde(rename = "OBS_MEDIA_STATE_STOPPED")]
    Stopped,
    /// All media in the play-list played.
    #[serde(rename = "OBS_MEDIA_STATE_ENDED")]
    Ended,
    /// Error occurred while trying to play the media.
    #[serde(rename = "OBS_MEDIA_STATE_ERROR")]
    Error,
    /// Unknown state.
    #[serde(other)]
    Unknown,
}

/// Response value for [`get_record_status`](crate::client::Recording::get_record_status).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordStatus {
    /// Whether the output is active.
    pub output_active: bool,
    /// Whether the output is paused.
    pub output_paused: bool,
    /// Current formatted time code string for the output.
    #[serde(deserialize_with = "crate::de::duration_timecode")]
    pub output_timecode: Duration,
    /// Current duration in milliseconds for the output.
    #[serde(deserialize_with = "crate::de::duration_millis")]
    pub output_duration: Duration,
    /// Number of bytes sent by the output.
    pub output_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OutputActive {
    /// New state of the stream output.
    pub output_active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OutputPaused {
    pub output_paused: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecordDirectory {
    /// Output directory.
    pub record_directory: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SavedReplayPath {
    pub saved_replay_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneItemId {
    /// Numeric ID of the scene item.
    pub scene_item_id: i64,
}

/// Response value for
/// [`get_scene_item_transform`](crate::client::SceneItems::get_scene_item_transform).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetSceneItemTransform {
    pub scene_item_transform: SceneItemTransform,
}

/// Response value for
/// [`get_scene_item_transform`](crate::client::SceneItems::get_scene_item_transform).
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneItemTransform {
    /// Base width (without scaling) of the source.
    pub source_width: f32,
    /// Base height (without scaling) of the source.
    pub source_height: f32,
    /// The x position of the source from the left.
    pub position_x: f32,
    /// The y position of the source from the top.
    pub position_y: f32,
    /// The clockwise rotation of the scene item in degrees around the point of alignment.
    pub rotation: f32,
    /// The x-scale factor of the source.
    pub scale_x: f32,
    /// The y-scale factor of the source.
    pub scale_y: f32,
    /// Scene item width (base source width multiplied by the horizontal scaling factor).
    pub width: f32,
    /// Scene item height (base source height multiplied by the vertical scaling factor).
    pub height: f32,
    /// The point on the source that the item is manipulated from.
    #[serde(deserialize_with = "crate::de::bitflags_u8")]
    pub alignment: Alignment,
    /// Type of bounding box.
    pub bounds_type: BoundsType,
    /// Alignment of the bounding box.
    #[serde(deserialize_with = "crate::de::bitflags_u8")]
    pub bounds_alignment: Alignment,
    /// Width of the bounding box.
    pub bounds_width: f32,
    /// Height of the bounding box.
    pub bounds_height: f32,
    /// The number of pixels cropped off the left of the source before scaling.
    pub crop_left: u32,
    /// The number of pixels cropped off the right of the source before scaling.
    pub crop_right: u32,
    /// The number of pixels cropped off the top of the source before scaling.
    pub crop_top: u32,
    /// The number of pixels cropped off the bottom of the source before scaling.
    pub crop_bottom: u32,
}

/// Response value for
/// [`get_scene_item_enabled`](crate::client::SceneItems::get_scene_item_enabled).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneItemEnabled {
    /// Whether the scene item is enabled.
    pub scene_item_enabled: bool,
}

/// Response value for [`get_scene_item_locked`](crate::client::SceneItems::get_scene_item_locked).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneItemLocked {
    /// Whether the scene item is locked.
    pub scene_item_locked: bool,
}

/// Response value for [`get_scene_item_index`](crate::client::SceneItems::get_scene_item_index).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneItemIndex {
    /// Index position of the scene item.
    pub scene_item_index: u32,
}

/// Response value for
/// [`get_scene_item_private_settings`](crate::client::SceneItems::get_scene_item_private_settings).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneItemSettings<T> {
    pub scene_item_settings: T,
}

/// Response value for
/// [`get_input_audio_sync_offset`](crate::client::Inputs::get_input_audio_sync_offset).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AudioSyncOffset {
    /// Audio sync offset in milliseconds.
    #[serde(deserialize_with = "crate::de::duration_millis")]
    pub input_audio_sync_offset: Duration,
}

/// Response value for
/// [`get_input_audio_monitor_type`](crate::client::Inputs::get_input_audio_monitor_type).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AudioMonitorType {
    /// Audio monitor type.
    pub monitor_type: MonitorType,
}

/// Response value for
/// [`get_input_properties_list_property_items`](crate::client::Inputs::get_input_properties_list_property_items).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListPropertyItems {
    /// Array of items in the list property.
    pub property_items: Vec<ListPropertyItem>,
}

/// Response value for
/// [`get_input_properties_list_property_items`](crate::client::Inputs::get_input_properties_list_property_items).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPropertyItem {
    /// Name of the item.
    pub item_name: String,
    /// Whether this item is enabled in the UI.
    pub item_enabled: bool,
    /// Content of the item, depending on what it represents.
    pub item_value: serde_json::Value,
}

/// Response value for [`get_scene_item_list`](crate::client::SceneItems::get_scene_item_list) and
/// [`get_group_scene_item_list`](crate::client::SceneItems::get_group_scene_item_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneItemList {
    /// Array of scene items in the scene or group.
    pub scene_items: Vec<SceneItem>,
}

/// Response value for [`get_scene_item_list`](crate::client::SceneItems::get_scene_item_list) and
/// [`get_group_scene_item_list`](crate::client::SceneItems::get_group_scene_item_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneItem {
    /// Identifier of the scene item.
    pub scene_item_id: i64,
    /// Positional index within a scene.
    pub scene_item_index: u32,
    /// Name of this source.
    pub source_name: String,
    /// The kind of source this item represents.
    pub source_type: SourceType,
    /// Kind of input. Only present if this is a [`SourceType::Input`].
    pub input_kind: Option<String>,
    /// Whether this item is a group. Only present if this is a [`SourceType::Scene`].
    pub is_group: Option<bool>,
}

/// Kind of source that is represented by a [`SceneItem`].
#[derive(Copy, Clone, Debug, Deserialize)]
pub enum SourceType {
    /// Input source from outside of OBS.
    #[serde(rename = "OBS_SOURCE_TYPE_INPUT")]
    Input,
    /// Filter applied to other items.
    #[serde(rename = "OBS_SOURCE_TYPE_FILTER")]
    Filter,
    /// Transition when switching scenes.
    #[serde(rename = "OBS_SOURCE_TYPE_TRANSITION")]
    Transition,
    /// Scene in OBS.
    #[serde(rename = "OBS_SOURCE_TYPE_SCENE")]
    Scene,
}

/// Response value for [`get_scene_list`](crate::client::Scenes::get_scene_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenes {
    /// Current program scene.
    pub current_program_scene_name: Option<String>,
    /// Current preview scene. [`None`] if not in studio mode.
    pub current_preview_scene_name: Option<String>,
    /// Array of scenes in OBS.
    pub scenes: Vec<Scene>,
}

/// Response value for [`get_scene_list`](crate::client::Scenes::get_scene_list) as part of
/// [`Scenes`].
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    /// Name of the scene.
    pub scene_name: String,
    /// Positional index in the list of scenes.
    pub scene_index: usize,
}

/// Response value for [`get_group_list`](crate::client::Scenes::get_group_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Groups {
    /// Array of group names.
    pub groups: Vec<String>,
}

/// Response value for
/// [`get_current_program_scene`](crate::client::Scenes::get_current_program_scene).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CurrentProgramScene {
    /// Current program scene.
    pub current_program_scene_name: String,
}

/// Response value for
/// [`get_current_preview_scene`](crate::client::Scenes::get_current_preview_scene).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CurrentPreviewScene {
    /// Current preview scene.
    pub current_preview_scene_name: String,
}

/// Response value for
/// [`get_scene_scene_transition_override`](crate::client::Scenes::get_scene_scene_transition_override).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneTransitionOverride {
    /// Name of the overridden scene transition.
    pub transition_name: Option<String>,
    /// Duration of the overridden scene transition.
    #[serde(deserialize_with = "crate::de::duration_millis_opt")]
    pub transition_duration: Option<Duration>,
}

/// Response value for [`get_source_active`](crate::client::Sources::get_source_active).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceActive {
    /// Whether the source is showing in program.
    pub video_active: bool,
    /// Whether the source is showing in the UI (preview, projector, properties).
    pub video_showing: bool,
}

/// Response value for [`get_source_screenshot`](crate::client::Sources::get_source_screenshot).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageData {
    /// Base64-encoded screenshot.
    pub image_data: String,
}

/// Response value for [`get_stream_status`](crate::client::Streaming::get_stream_status).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamStatus {
    /// Whether the output is active.
    pub output_active: bool,
    /// Whether the output is currently reconnecting.
    pub output_reconnecting: bool,
    /// Current time code for the output.
    #[serde(deserialize_with = "crate::de::duration_timecode")]
    pub output_timecode: Duration,
    /// Current duration for the output.
    #[serde(deserialize_with = "crate::de::duration_millis")]
    pub output_duration: Duration,
    /// Number of bytes sent by the output.
    pub output_bytes: u64,
    /// Number of frames skipped by the output's process.
    pub output_skipped_frames: u32,
    /// Total number of frames delivered by the output's process.
    pub output_total_frames: u32,
}

/// Response value for
/// [`get_transition_kind_list`](crate::client::Transitions::get_transition_kind_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransitionKinds {
    /// Array of transition kinds.
    pub transition_kinds: Vec<String>,
}

/// Response value for
/// [`get_scene_transition_list`](crate::client::Transitions::get_scene_transition_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneTransitionList {
    /// Name of the current scene transition.
    pub current_scene_transition_name: Option<String>,
    /// Kind of the current scene transition.
    pub current_scene_transition_kind: Option<String>,
    /// Array of transitions.
    pub transitions: Vec<Transition>,
}

/// Response value for
/// [`get_scene_transition_list`](crate::client::Transitions::get_scene_transition_list) as part of
/// [`SceneTransitionList`].
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    /// Name of the transition.
    pub transition_name: String,
    /// Kind of the transition.
    pub transition_kind: String,
    /// Whether the transition uses a fixed (non-configurable) duration.
    pub transition_fixed: bool,
    /// Whether the transition supports being configured.
    pub transition_configurable: bool,
}

/// Response value for
/// [`get_current_scene_transition`](crate::client::Transitions::get_current_scene_transition).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentSceneTransition {
    /// Name of the transition.
    pub transition_name: String,
    /// Kind of the transition.
    pub transition_kind: String,
    /// Whether the transition uses a fixed (non-configurable) duration.
    pub transition_fixed: bool,
    /// Configured transition duration in milliseconds.
    #[serde(deserialize_with = "crate::de::duration_millis_opt")]
    pub transition_duration: Option<Duration>,
    /// Whether the transition supports being configured.
    pub transition_configurable: bool,
    /// Object of settings for the transition.
    pub transition_settings: Option<serde_json::Value>,
}

/// Response value for
/// [`get_current_scene_transition_cursor`](crate::client::Transitions::get_current_scene_transition_cursor).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransitionCursor {
    /// Cursor position, between `0.0` and `1.0`.
    pub transition_cursor: f32,
}

/// Response value for [`get_monitor_list`](crate::client::Ui::get_monitor_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MonitorList {
    pub monitors: Vec<Monitor>,
}

/// Response value for [`get_monitor_list`](crate::client::Ui::get_monitor_list).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    /// Name of this monitor.
    pub monitor_name: String,
    /// Pixel width.
    pub monitor_width: u16,
    /// Pixel height.
    pub monitor_height: u16,
    /// Horizontal position on the screen.
    pub monitor_position_x: u16,
    /// Vertical position on the screen.
    pub monitor_position_y: u16,
}
