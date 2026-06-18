use std::ffi::{c_char, c_void};
use std::os::raw::c_float;

pub type DWORD = u32;
pub type QWORD = u64;
pub type HSTREAM = DWORD;
pub type HPLUGIN = DWORD;
pub type BOOL = i32;

pub const BASS_OK: DWORD = 0;
pub const BASS_ERROR_MEM: DWORD = 1;
pub const BASS_ERROR_FILEOPEN: DWORD = 2;
pub const BASS_ERROR_DRIVER: DWORD = 3;
pub const BASS_ERROR_BUFLOST: DWORD = 4;
pub const BASS_ERROR_HANDLE: DWORD = 5;
pub const BASS_ERROR_FORMAT: DWORD = 6;
pub const BASS_ERROR_POSITION: DWORD = 7;
pub const BASS_ERROR_INIT: DWORD = 8;
pub const BASS_ERROR_BASSINIT: DWORD = 9;
pub const BASS_ERROR_START: DWORD = 10;
pub const BASS_ERROR_SSL: DWORD = 11;
pub const BASS_ERROR_ALREADY: DWORD = 14;
pub const BASS_ERROR_NOTAUDIO: DWORD = 17;
pub const BASS_ERROR_NOCHAN: DWORD = 18;
pub const BASS_ERROR_ILLTYPE: DWORD = 19;
pub const BASS_ERROR_ILLPARAM: DWORD = 20;
pub const BASS_ERROR_NO3D: DWORD = 21;
pub const BASS_ERROR_NOEAX: DWORD = 22;
pub const BASS_ERROR_DEVICE: DWORD = 23;
pub const BASS_ERROR_NOPLAY: DWORD = 24;
pub const BASS_ERROR_FREQ: DWORD = 25;
pub const BASS_ERROR_NOTFILE: DWORD = 26;
pub const BASS_ERROR_NOHW: DWORD = 29;
pub const BASS_ERROR_EMPTY: DWORD = 31;
pub const BASS_ERROR_NONET: DWORD = 32;
pub const BASS_ERROR_CREATE: DWORD = 33;
pub const BASS_ERROR_NOFX: DWORD = 34;
pub const BASS_ERROR_NOTAVAIL: DWORD = 37;
pub const BASS_ERROR_DECODE: DWORD = 38;
pub const BASS_ERROR_DX: DWORD = 39;
pub const BASS_ERROR_TIMEOUT: DWORD = 40;
pub const BASS_ERROR_FILEFORM: DWORD = 41;
pub const BASS_ERROR_SPEAKER: DWORD = 42;
pub const BASS_ERROR_VERSION: DWORD = 43;
pub const BASS_ERROR_CODEC: DWORD = 44;
pub const BASS_ERROR_ENDED: DWORD = 45;
pub const BASS_ERROR_BUSY: DWORD = 46;
pub const BASS_ERROR_UNSTREAMED: DWORD = 47;
pub const BASS_ERROR_PROTOCOL: DWORD = 48;
pub const BASS_ERROR_DENIED: DWORD = 49;
pub const BASS_ERROR_UNKNOWN: DWORD = -1i32 as DWORD;

pub const BASS_CONFIG_DEV_NONSTOP: DWORD = 24;
pub const BASS_CONFIG_DEV_DEFAULT: DWORD = 50;

pub const BASS_ACTIVE_STOPPED: DWORD = 0;
pub const BASS_ACTIVE_PLAYING: DWORD = 1;
pub const BASS_ACTIVE_STALLED: DWORD = 2;
pub const BASS_ACTIVE_PAUSED: DWORD = 3;

pub const BASS_ATTRIB_FREQ: DWORD = 1;
pub const BASS_ATTRIB_VOL: DWORD = 2;
pub const BASS_ATTRIB_PAN: DWORD = 3;

pub const BASS_SYNC_END: DWORD = 2;
pub const BASS_SYNC_POS: DWORD = 0;

pub const BASS_STREAM_AUTOFREE: DWORD = 0x40000;
pub const BASS_UNICODE: DWORD = 0x80000000;

pub type SYNCPROC = unsafe extern "C" fn(handle: HSTREAM, channel: DWORD, data: DWORD, user: *mut c_void);

pub fn bass_error_name(code: DWORD) -> &'static str {
    match code {
        BASS_OK => "BASS_OK",
        BASS_ERROR_MEM => "BASS_ERROR_MEM",
        BASS_ERROR_FILEOPEN => "BASS_ERROR_FILEOPEN",
        BASS_ERROR_DRIVER => "BASS_ERROR_DRIVER",
        BASS_ERROR_FORMAT => "BASS_ERROR_FORMAT",
        BASS_ERROR_INIT => "BASS_ERROR_INIT",
        BASS_ERROR_HANDLE => "BASS_ERROR_HANDLE",
        BASS_ERROR_NOCHAN => "BASS_ERROR_NOCHAN",
        BASS_ERROR_ILLPARAM => "BASS_ERROR_ILLPARAM",
        BASS_ERROR_NOTFILE => "BASS_ERROR_NOTFILE",
        BASS_ERROR_FILEFORM => "BASS_ERROR_FILEFORM",
        BASS_ERROR_CODEC => "BASS_ERROR_CODEC",
        BASS_ERROR_ENDED => "BASS_ERROR_ENDED",
        BASS_ERROR_DECODE => "BASS_ERROR_DECODE",
        BASS_ERROR_NOTAVAIL => "BASS_ERROR_NOTAVAIL",
        BASS_ERROR_UNKNOWN => "BASS_ERROR_UNKNOWN",
        _ => "BASS_ERROR_OTHER",
    }
}

pub fn bass_error_message(code: DWORD) -> String {
    match code {
        BASS_OK => "成功".to_string(),
        BASS_ERROR_FILEOPEN => "无法打开文件".to_string(),
        BASS_ERROR_FORMAT => "文件格式不支持".to_string(),
        BASS_ERROR_CODEC => "编解码器不支持，可能需要安装插件".to_string(),
        BASS_ERROR_FILEFORM => "文件格式无法识别或编码不支持".to_string(),
        BASS_ERROR_DECODE => "解码失败，文件可能已损坏".to_string(),
        BASS_ERROR_DEVICE => "音频设备不可用（可能因蓝牙耳机切换）".to_string(),
        BASS_ERROR_INIT => "音频设备未初始化".to_string(),
        BASS_ERROR_START => "音频输出未启动".to_string(),
        BASS_ERROR_HANDLE => "播放通道已失效".to_string(),
        BASS_ERROR_BUFLOST => "音频缓冲丢失".to_string(),
        _ => format!("BASS 错误 {}", code),
    }
}
