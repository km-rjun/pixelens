//! Daemon-wide state shared between the IPC server and the dispatcher.
//!
//! In v1 the state is small: a snapshot of the detected display server
//! and an optional capture pipeline. The pipeline may be absent if
//! `slurp` / `grim` aren't installed; in that case `pixelens grab`
//! returns a clear `MissingTool` error and the daemon keeps running
//! for the other commands.

use pixelens_capture::{DisplayServer, GrabPipeline};

pub struct DaemonState {
    pub display: DisplayServer,
    /// `None` when the pipeline failed to construct (e.g. slurp/grim
    /// missing). All other commands are unaffected; only `grab`
    /// surfaces this.
    pub pipeline: Option<GrabPipeline>,
}

impl DaemonState {
    pub fn new(display: DisplayServer, pipeline: Option<GrabPipeline>) -> Self {
        Self { display, pipeline }
    }
}
