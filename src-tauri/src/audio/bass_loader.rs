use crate::audio::bass_ffi::*;
use crate::errors::{AppError, AppResult};
use libloading::{Library, Symbol};
use std::ffi::{c_void, CString, OsStr};
use std::os::raw::c_float;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::Manager;

type BASS_InitFn = unsafe extern "C" fn(device: i32, freq: DWORD, flags: DWORD, win: *mut c_void, clsid: *const c_void) -> BOOL;
type BASS_FreeFn = unsafe extern "C" fn() -> BOOL;
type BASS_GetVersionFn = unsafe extern "C" fn() -> DWORD;
type BASS_ErrorGetCodeFn = unsafe extern "C" fn() -> i32;
type BASS_PluginLoadFn = unsafe extern "C" fn(file: *const u16, flags: DWORD) -> HPLUGIN;
type BASS_PluginFreeFn = unsafe extern "C" fn(handle: HPLUGIN) -> BOOL;
type BASS_StreamCreateFileFn = unsafe extern "C" fn(mem: BOOL, file: *const u16, offset: QWORD, length: QWORD, flags: DWORD) -> HSTREAM;
type BASS_StreamFreeFn = unsafe extern "C" fn(handle: HSTREAM) -> BOOL;
type BASS_ChannelPlayFn = unsafe extern "C" fn(handle: DWORD, restart: BOOL) -> BOOL;
type BASS_ChannelPauseFn = unsafe extern "C" fn(handle: DWORD) -> BOOL;
type BASS_ChannelStopFn = unsafe extern "C" fn(handle: DWORD) -> BOOL;
type BASS_ChannelSetPositionFn = unsafe extern "C" fn(handle: DWORD, pos: QWORD, mode: DWORD) -> BOOL;
type BASS_ChannelGetPositionFn = unsafe extern "C" fn(handle: DWORD, mode: DWORD) -> QWORD;
type BASS_ChannelGetLengthFn = unsafe extern "C" fn(handle: DWORD, mode: DWORD) -> QWORD;
type BASS_ChannelBytes2SecondsFn = unsafe extern "C" fn(handle: DWORD, pos: QWORD) -> f64;
type BASS_ChannelSeconds2BytesFn = unsafe extern "C" fn(handle: DWORD, pos: f64) -> QWORD;
type BASS_ChannelSetAttributeFn = unsafe extern "C" fn(handle: DWORD, attrib: DWORD, value: c_float) -> BOOL;
type BASS_ChannelGetAttributeFn = unsafe extern "C" fn(handle: DWORD, attrib: DWORD, value: *mut c_float) -> BOOL;
type BASS_ChannelIsActiveFn = unsafe extern "C" fn(handle: DWORD) -> DWORD;
type BASS_ChannelSetSyncFn = unsafe extern "C" fn(handle: DWORD, type_: DWORD, param: QWORD, proc: SYNCPROC, user: *mut c_void) -> DWORD;
type BASS_SetConfigFn = unsafe extern "C" fn(option: DWORD, value: DWORD) -> BOOL;
type BASS_StartFn = unsafe extern "C" fn() -> BOOL;

pub struct BassLoader {
    lib: Library,
    audio_core_dir: PathBuf,
}

pub struct BassFunctions {
    pub bass_init: BASS_InitFn,
    pub bass_free: BASS_FreeFn,
    pub bass_get_version: BASS_GetVersionFn,
    pub bass_error_get_code: BASS_ErrorGetCodeFn,
    pub bass_plugin_load: BASS_PluginLoadFn,
    pub bass_plugin_free: BASS_PluginFreeFn,
    pub bass_stream_create_file: BASS_StreamCreateFileFn,
    pub bass_stream_free: BASS_StreamFreeFn,
    pub bass_channel_play: BASS_ChannelPlayFn,
    pub bass_channel_pause: BASS_ChannelPauseFn,
    pub bass_channel_stop: BASS_ChannelStopFn,
    pub bass_channel_set_position: BASS_ChannelSetPositionFn,
    pub bass_channel_get_position: BASS_ChannelGetPositionFn,
    pub bass_channel_get_length: BASS_ChannelGetLengthFn,
    pub bass_channel_bytes2seconds: BASS_ChannelBytes2SecondsFn,
    pub bass_channel_seconds2bytes: BASS_ChannelSeconds2BytesFn,
    pub bass_channel_set_attribute: BASS_ChannelSetAttributeFn,
    pub bass_channel_get_attribute: BASS_ChannelGetAttributeFn,
    pub bass_channel_is_active: BASS_ChannelIsActiveFn,
    pub bass_channel_set_sync: BASS_ChannelSetSyncFn,
    pub bass_set_config: Option<BASS_SetConfigFn>,
    pub bass_start: Option<BASS_StartFn>,
}

unsafe impl Send for BassFunctions {}
unsafe impl Sync for BassFunctions {}

impl BassLoader {
    pub fn load(audio_core_dir: &Path) -> AppResult<Self> {
        let bass_dll = audio_core_dir.join("bass.dll");
        if !bass_dll.exists() {
            return Err(AppError::AudioCoreMissing(format!(
                "bass.dll 未找到，期望路径: {}",
                bass_dll.display()
            )));
        }

        // Add audio_core dir to DLL search path
        let wide: Vec<u16> = OsStr::new(&audio_core_dir.to_string_lossy().as_ref())
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        unsafe {
            windows_sys_add_dll_directory(&wide);
        }

        let lib = unsafe { Library::new(&bass_dll) }.map_err(|e| {
            AppError::AudioCoreLoadFailed(format!("加载 bass.dll 失败: {}", e))
        })?;

        Ok(Self {
            lib,
            audio_core_dir: audio_core_dir.to_path_buf(),
        })
    }

    pub fn load_functions(&self) -> AppResult<BassFunctions> {
        unsafe {
            let bass_init = self.load_sym::<BASS_InitFn>("BASS_Init")?;
            let bass_free = self.load_sym::<BASS_FreeFn>("BASS_Free")?;
            let bass_get_version = self.load_sym::<BASS_GetVersionFn>("BASS_GetVersion")?;
            let bass_error_get_code = self.load_sym::<BASS_ErrorGetCodeFn>("BASS_ErrorGetCode")?;
            let bass_plugin_load = self.load_sym::<BASS_PluginLoadFn>("BASS_PluginLoad")?;
            let bass_plugin_free = self.load_sym::<BASS_PluginFreeFn>("BASS_PluginFree")?;
            let bass_stream_create_file = self.load_sym::<BASS_StreamCreateFileFn>("BASS_StreamCreateFile")?;
            let bass_stream_free = self.load_sym::<BASS_StreamFreeFn>("BASS_StreamFree")?;
            let bass_channel_play = self.load_sym::<BASS_ChannelPlayFn>("BASS_ChannelPlay")?;
            let bass_channel_pause = self.load_sym::<BASS_ChannelPauseFn>("BASS_ChannelPause")?;
            let bass_channel_stop = self.load_sym::<BASS_ChannelStopFn>("BASS_ChannelStop")?;
            let bass_channel_set_position = self.load_sym::<BASS_ChannelSetPositionFn>("BASS_ChannelSetPosition")?;
            let bass_channel_get_position = self.load_sym::<BASS_ChannelGetPositionFn>("BASS_ChannelGetPosition")?;
            let bass_channel_get_length = self.load_sym::<BASS_ChannelGetLengthFn>("BASS_ChannelGetLength")?;
            let bass_channel_bytes2seconds = self.load_sym::<BASS_ChannelBytes2SecondsFn>("BASS_ChannelBytes2Seconds")?;
            let bass_channel_seconds2bytes = self.load_sym::<BASS_ChannelSeconds2BytesFn>("BASS_ChannelSeconds2Bytes")?;
            let bass_channel_set_attribute = self.load_sym::<BASS_ChannelSetAttributeFn>("BASS_ChannelSetAttribute")?;
            let bass_channel_get_attribute = self.load_sym::<BASS_ChannelGetAttributeFn>("BASS_ChannelGetAttribute")?;
            let bass_channel_is_active = self.load_sym::<BASS_ChannelIsActiveFn>("BASS_ChannelIsActive")?;
            let bass_channel_set_sync = self.load_sym::<BASS_ChannelSetSyncFn>("BASS_ChannelSetSync")?;
            let bass_set_config = self.load_sym::<BASS_SetConfigFn>("BASS_SetConfig").ok();
            let bass_start = self.load_sym::<BASS_StartFn>("BASS_Start").ok();

            Ok(BassFunctions {
                bass_init,
                bass_free,
                bass_get_version,
                bass_error_get_code,
                bass_plugin_load,
                bass_plugin_free,
                bass_stream_create_file,
                bass_stream_free,
                bass_channel_play,
                bass_channel_pause,
                bass_channel_stop,
                bass_channel_set_position,
                bass_channel_get_position,
                bass_channel_get_length,
                bass_channel_bytes2seconds,
                bass_channel_seconds2bytes,
                bass_channel_set_attribute,
                bass_channel_get_attribute,
                bass_channel_is_active,
                bass_channel_set_sync,
                bass_set_config,
                bass_start,
            })
        }
    }

    unsafe fn load_sym<T: Copy>(&self, name: &str) -> AppResult<T> {
        let c_name = CString::new(name).map_err(|e| {
            AppError::AudioCoreLoadFailed(format!("无效符号名 {}: {}", name, e))
        })?;
        let sym: Symbol<T> = self.lib.get(c_name.as_bytes()).map_err(|e| {
            AppError::AudioCoreLoadFailed(format!("符号 {} 未找到: {}", name, e))
        })?;
        Ok(*sym.into_raw())
    }

    pub fn audio_core_dir(&self) -> &Path {
        &self.audio_core_dir
    }
}

unsafe fn windows_sys_add_dll_directory(wide_path: &[u16]) {
    // Use SetDllDirectoryW to add directory to search path
    #[link(name = "kernel32")]
    extern "system" {
        fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
    }
    SetDllDirectoryW(wide_path.as_ptr());
}

pub fn resolve_audio_core_dir(app_handle: &tauri::AppHandle) -> AppResult<PathBuf> {
    // Try packaged resource directory first
    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        let audio_core = resource_dir.join("audio_core");
        if audio_core.join("bass.dll").exists() {
            tracing::info!("Found audio_core in resource dir: {}", audio_core.display());
            return Ok(audio_core);
        }
    }

    // Try EXE sibling directory
    if let Ok(exe_dir) = std::env::current_exe() {
        let exe_parent = exe_dir.parent().unwrap_or(Path::new("."));
        let audio_core = exe_parent.join("resources").join("audio_core");
        if audio_core.join("bass.dll").exists() {
            tracing::info!("Found audio_core in exe sibling: {}", audio_core.display());
            return Ok(audio_core);
        }
    }

    // Dev: src-tauri/resources/audio_core
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok();
    if let Some(ref dir) = manifest_dir {
        let audio_core = PathBuf::from(dir).join("resources").join("audio_core");
        if audio_core.join("bass.dll").exists() {
            tracing::info!("Found audio_core in src-tauri/resources: {}", audio_core.display());
            return Ok(audio_core);
        }
    }

    // Dev: vendor/audio_core/x64 (relative to project root)
    if let Some(ref dir) = manifest_dir {
        let project_root = PathBuf::from(dir).parent().map(|p| p.to_path_buf());
        if let Some(root) = project_root {
            let audio_core = root.join("vendor").join("audio_core").join("x64");
            if audio_core.join("bass.dll").exists() {
                tracing::info!("Found audio_core in vendor: {}", audio_core.display());
                return Ok(audio_core);
            }
        }
    }

    Err(AppError::AudioCoreMissing(
        "未找到 audio_core 目录，请确保 bass.dll 存在于 resources/audio_core 或 vendor/audio_core/x64".to_string(),
    ))
}
