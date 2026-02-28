//! CUE objects dealocation

unsafe extern "C" {
    /// Releases the resource identified by `handle`.
    pub fn cue_free(handle: usize);
}
