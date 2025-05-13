/// Internally tracked state of the export process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExportState {
    /// Setup phase: store camera state, switch `RenderTarget`, and move the camera to the first tile position.
    PrepareTargetAndCamera,
    /// To prevent artefacts, we have to give Bevy the chance to render a frame without capturing.
    SkipFirstFrame,
    /// Each frame the camera is moved to the next tile position while readbacks are being captured.
    Capturing,
    /// Camera has moved to all positions, we are waiting for the readbacks to finish.
    AwaitReadbacks,
    /// All frames have been captured and read, we offload the processing to an async task.
    ProcessFrames,
    /// Clean-up phase: restore the camera state, switch `RenderTarget`,
    /// and move the camera to a default position (usually 0x0).
    Cleanup,
}
