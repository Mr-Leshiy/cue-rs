//! CUE objects dealocation

#[allow(dead_code)]
unsafe extern "C" {
    /// Releases the resource identified by `handle`.
    pub(crate) fn cue_free(handle: usize);

    /// Releases all resources identified by the null-terminated array of handles.
    pub(crate) fn cue_free_all(handles: *mut usize);

    /// Frees memory allocated by the C library (libc).
    pub(crate) fn libc_free(ptr: *mut core::ffi::c_void);
}
