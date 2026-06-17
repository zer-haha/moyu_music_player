// 防止 Windows 控制台程序打开额外的控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    music_player_lib::run()
}
