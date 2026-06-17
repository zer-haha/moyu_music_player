use crate::audio::AudioEngine;
use crate::db::connection::Database;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub audio: AudioEngine,
    pub db: Database,
    pub scan_tokens: Mutex<HashMap<String, Arc<AtomicBool>>>,
}
