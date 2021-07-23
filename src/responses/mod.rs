//! All responses that can be received from the API.

pub use semver::Version as SemVerVersion;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Debug, Deserialize)]
#[serde(tag = "messageType")]
pub(crate) enum ServerMessage {
    /// First message sent from the server immediately on client connection. Contains authentication
    /// information if auth is required. Also contains RPC version for version negotiation.
    #[serde(rename_all = "camelCase")]
    Hello {
        obs_web_socket_version: SemVerVersion,
        /// Version number which gets incremented on each **breaking change** to the obs-websocket
        /// protocol.
        rpc_version: u32,
        authentication: Option<Authentication>,
    },
    /// The identify request was received and validated, and the connection is now ready for normal
    /// operation.
    #[serde(rename_all = "camelCase")]
    Identified {
        /// The RPC version to be used.
        negotiated_rpc_version: u32,
    },
    /// An event coming from OBS has occurred. Eg scene switched, source muted.
    #[cfg(feature = "events")]
    #[serde(rename_all = "camelCase")]
    Event(crate::events::Event),
    /// `obs-websocket` is responding to a request coming from a client.
    #[serde(rename_all = "camelCase")]
    RequestResponse {
        request_id: String,
        request_status: Status,
        #[serde(default)]
        response_data: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    RequestBatchResponse {
        request_id: String,
        results: Vec<serde_json::Value>,
    },
}

#[derive(Debug, Deserialize)]
pub(crate) struct Authentication {
    pub challenge: String,
    pub salt: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Status {
    pub result: bool,
    pub code: StatusCode,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u16)]
pub enum StatusCode {
    Unknown = 0,
    /// For internal use to signify a successful parameter check.
    NoError = 10,
    Success = 100,
    /// The `requestType` field is missing from the request data.
    MissingRequestType = 203,
    /// The request type is invalid (does not exist).
    UnknownRequestType = 204,
    /// Generic error code (comment is expected to be provided).
    GenericError = 205,
    /// A required request parameter is missing.
    MissingRequestParameter = 300,
    /// The request does not have a valid requestData object.
    MissingRequestData = 301,
    /// Generic invalid request parameter message.
    InvalidRequestParameter = 400,
    /// A request parameter has the wrong data type.
    InvalidRequestParameterDataType = 401,
    /// A request parameter (float or int) is out of valid range.
    RequestParameterOutOfRange = 402,
    /// A request parameter (string or array) is empty and cannot be.
    RequestParameterEmpty = 403,
    /// An output is running and cannot be in order to perform the request (generic).
    OutputRunning = 500,
    /// An output is not running and should be.
    OutputNotRunning = 501,
    /// Stream is running and cannot be.
    StreamRunning = 502,
    /// Stream is not running and should be.
    StreamNotRunning = 503,
    /// Record is running and cannot be.
    RecordRunning = 504,
    /// Record is not running and should be.
    RecordNotRunning = 505,
    /// Record is paused and cannot be.
    RecordPaused = 506,
    /// Replay buffer is running and cannot be.
    ReplayBufferRunning = 507,
    /// Replay buffer is not running and should be.
    ReplayBufferNotRunning = 508,
    /// Replay buffer is disabled and cannot be.
    ReplayBufferDisabled = 509,
    /// Studio mode is active and cannot be.
    StudioModeActive = 510,
    /// Studio mode is not active and should be.
    StudioModeNotActive = 511,
    /// Virtualcam is running and cannot be.
    VirtualCamRunning = 512,
    /// Virtualcam is not running and should be.
    VirtualCamNotRunning = 513,
    /// The specified source (obs_source_t) was of the invalid type (Eg. input instead of scene).
    InvalidSourceType = 600,
    /// The specified source (obs_source_t) was not found (generic for input, filter, transition,
    /// scene).
    SourceNotFound = 601,
    /// The specified source (obs_source_t) already exists. Applicable to inputs, filters,
    /// transitions, scenes.
    SourceAlreadyExists = 602,
    /// The specified input (obs_source_t-OBS_SOURCE_TYPE_FILTER) was not found.
    InputNotFound = 603,
    /// The specified input (obs_source_t-OBS_SOURCE_TYPE_INPUT) had the wrong kind.
    InvalidInputKind = 604,
    /// The specified filter (obs_source_t-OBS_SOURCE_TYPE_FILTER) was not found.
    FilterNotFound = 605,
    /// The specified transition (obs_source_t-OBS_SOURCE_TYPE_TRANSITION) was not found.
    TransitionNotFound = 606,
    /// The specified transition (obs_source_t-OBS_SOURCE_TYPE_TRANSITION) does not support setting
    /// its position (transition is of fixed type).
    TransitionDurationFixed = 607,
    /// The specified scene (obs_source_t-OBS_SOURCE_TYPE_SCENE), (obs_scene_t) was not found.
    SceneNotFound = 608,
    /// The specified scene item (obs_sceneitem_t) was not found.
    SceneItemNotFound = 609,
    /// The specified scene collection was not found.
    SceneCollectionNotFound = 610,
    /// The specified profile was not found.
    ProfileNotFound = 611,
    /// The specified output (obs_output_t) was not found.
    OutputNotFound = 612,
    /// The specified encoder (obs_encoder_t) was not found.
    EncoderNotFound = 613,
    /// The specified service (obs_service_t) was not found.
    ServiceNotFound = 614,
    /// The specified hotkey was not found.
    HotkeyNotFound = 615,
    /// The specified directory was not found.
    DirectoryNotFound = 616,
    /// The specified config item (config_t) was not found. Could be section or parameter name.
    ConfigParameterNotFound = 617,
    /// The specified property (obs_properties_t) was not found.
    PropertyNotFound = 618,
    /// The specififed key (OBS_KEY_*) was not found.
    KeyNotFound = 619,
    /// Processing the request failed unexpectedly.
    RequestProcessingFailed = 700,
    /// Starting the Output failed.
    OutputStartFailed = 701,
    /// Duplicating the scene item failed.
    SceneItemDuplicationFailed = 702,
    /// Rendering the screenshot failed.
    ScreenshotRenderFailed = 703,
    /// Encoding the screenshot failed.
    ScreenshotEncodeFailed = 704,
    /// Saving the screenshot failed.
    ScreenshotSaveFailed = 705,
    /// Creating the directory failed.
    DirectoryCreationFailed = 706,
    /// The combination of request parameters cannot be used to perform an action.
    CannotAct = 707,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneCollections {
    pub current_scene_collection_name: String,
    pub scene_collections: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profiles {
    pub current_profile_name: String,
    pub profiles: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileParameter {
    pub parameter_value: Option<String>,
    pub default_parameter_value: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub obs_version: SemVerVersion,
    pub obs_web_socket_version: SemVerVersion,
    pub rpc_version: u32,
    pub available_requests: Vec<String>,
    pub supported_image_formats: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Hotkeys {
    pub hotkeys: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StudioModeEnabled {
    pub studio_mode_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Inputs {
    pub inputs: Vec<Input>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub input_name: String,
    pub input_kind: String,
    pub unversioned_input_kind: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputKinds {
    pub input_kinds: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultInputSettings {
    pub default_input_settings: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSettings {
    pub input_settings: serde_json::Value,
    pub input_kind: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMuted {
    pub input_muted: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputVolume {
    pub input_volume_mul: f64,
    pub input_volume_db: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneItemId {
    pub scene_item_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenes {
    pub current_program_scene_name: String,
    pub current_preview_scene_name: Option<String>,
    pub scenes: Vec<Scene>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub scene_name: String,
    pub scene_index: u64,
    pub is_group: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CurrentProgramScene {
    pub current_program_scene_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CurrentPreviewScene {
    pub current_preview_scene_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceActive {
    pub video_active: bool,
    pub video_showing: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageData {
    pub image_data: String,
}
