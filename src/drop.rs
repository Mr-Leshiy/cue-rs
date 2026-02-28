//! CUE objects dealocation

unsafe extern "C" {
    /// Releases the resource identified by `handle`.
    pub fn cue_free(handle: usize);

    /// Releases all resources identified by the null-terminated array of handles.
    pub fn cue_free_all(handles: *mut usize);

    /// Frees memory allocated by the C library (libc).
    pub fn libc_free(ptr: *mut core::ffi::c_void);
}
