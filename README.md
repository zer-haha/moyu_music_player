# 墨鱼听歌 — Windows 本地音乐播放器

一个基于 Tauri 2 + Vue 3 + Rust + BASS Audio Library 的轻量级 Windows 本地单机音乐播放器。不需要安装 mpv、FFmpeg 或任何外部播放器内核，双击 EXE 即可运行。

---

## 一、技术栈

### 前端

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue 3 | ^3.5.13 | 响应式 UI 框架 |
| TypeScript | ~5.7.3 | 类型安全 |
| Vite | ^6.3.5 | 构建工具 + 开发服务器 |
| Pinia | ^2.3.1 | 状态管理 |
| @tauri-apps/api | ^2.5.0 | Tauri 前端 API（invoke / event / window） |
| @tauri-apps/plugin-dialog | ^2.2.2 | 文件/文件夹选择对话框 |

### 后端（Rust）

| 技术 | 版本 | 用途 |
|------|------|------|
| Tauri | 2 | 桌面应用框架（tray-icon 功能） |
| tauri-plugin-dialog | 2 | 原生对话框 |
| BASS Audio Library | — | 核心音频播放引擎（动态加载 DLL） |
| libloading | 0.8 | 运行时动态加载 bass.dll |
| rusqlite | 0.32 (bundled) | SQLite 数据库（内置编译，无需系统 DLL） |
| lofty | 0.22 | 读取音乐元数据标签（ID3/Vorbis/MP4 等） |
| walkdir | 2 | 递归扫描文件夹 |
| crossbeam-channel | 0.5 | 音频线程命令队列 |
| parking_lot | 0.12 | 高性能 Mutex |
| chrono | 0.4 | 时间处理 |
| tracing | 0.1 | 结构化日志 |
| tracing-subscriber | 0.3 | 日志输出 |
| tracing-appender | 0.2 | 日志文件滚动 |
| thiserror | 2 | 错误类型派生 |
| tokio | 1 (full) | 异步运行时 |
| uuid | 1 (v4) | 扫描任务 ID 生成 |
| serde / serde_json | 1 | 序列化/反序列化 |

### 构建工具

| 工具 | 说明 |
|------|------|
| cargo / rustc | Rust 编译器（edition 2021） |
| npm | 前端包管理 |
| NSIS | Windows 安装包打包 |
| tauri-build | Tauri 资源打包 |

---

## 二、功能列表

### 音频播放

- **BASS 音频引擎**：通过 `libloading` 动态加载 `bass.dll`，不依赖系统 PATH
- **多格式支持**：MP3、MP2、MP1、WAV、AIFF、AIF、OGG、FLAC、M4A（AAC/ALAC）、AAC、MP4、ALAC、OPUS、APE、WMA、WV、TTA、MKA
- **插件化解码**：6 个 BASS 插件 DLL 分别处理不同格式
  - `bassflac.dll` — FLAC
  - `bass_aac.dll` — AAC / M4A (AAC)
  - `bassalac.dll` — ALAC / M4A (Apple Lossless)
  - `bassopus.dll` — Opus
  - `bassape.dll` — APE (Monkey's Audio)
  - `basswma.dll` — WMA
- **播放控制**：播放、暂停、恢复、停止、上一首、下一首
- **进度条拖动**：鼠标拖动时只更新 UI 预览，松手后一次性 seek，不卡顿
- **音量控制**：0.0–1.0 范围调节，支持静音
- **播放模式**：列表循环、单曲循环、顺序播放、随机播放

### 播放列表管理

- **多播放列表**：支持创建、重命名、删除、清空播放列表
- **默认播放列表**：首次启动自动创建，不可删除
- **拖动排序**：左侧播放列表支持鼠标拖动改变顺序
- **歌曲管理**：从列表移除、从歌库删除、去重
- **右键菜单**：播放列表和歌曲均有右键操作菜单
- **搜索过滤**：实时搜索当前列表（标题/歌手/专辑/路径）

### 文件导入

- **文件选择器导入**：通过对话框选择音乐文件
- **文件夹递归扫描**：选择文件夹后自动递归扫描所有子目录
- **拖放导入**：拖放文件/文件夹到窗口
  - 拖到歌曲列表区域 → 加入当前播放列表
  - 拖到左侧指定播放列表 → 加入该列表
  - 拖到左侧空白区域 → 按文件夹名创建新播放列表
- **自动去重**：基于路径规范化（小写 + 统一分隔符）去重
- **元数据读取**：使用 lofty 读取标题、艺术家、专辑、时长等标签

### 数据持久化

- **SQLite 数据库**：所有播放列表、歌曲、设置持久化存储
- **WAL 模式**：启用 Write-Ahead Logging 提升并发性能
- **Schema 迁移**：版本化迁移系统，自动建表
- **路径自适应**：优先使用 EXE 同级 `data/` 目录，回退到 AppData
- **状态恢复**：重启后恢复音量、播放模式、当前播放列表、上次选中歌曲

### 界面交互

- **自定义标题栏**：无边框窗口，自定义标题栏可拖动
- **系统托盘**：关闭时弹出确认对话框（退出/最小化到托盘/取消）
- **迷你模式**：切换到 380×80 迷你播放器
- **窗口置顶**：一键切换窗口置顶
- **虚拟列表**：歌曲列表使用虚拟滚动，3000+ 首流畅滚动
- **深色/浅色主题**：支持切换，自动保存
- **禁用默认右键菜单**：屏蔽 WebView2 自带的右键菜单

---

## 三、架构设计

### 整体架构

```
┌─────────────────────────────────────────────────┐
│                   Vue 3 前端                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │ App.vue  │ │ Stores   │ │ Components       │ │
│  │ (入口)    │ │ (Pinia)  │ │ (Header/Sidebar/ │ │
│  │          │ │          │ │  TrackList/...)  │ │
│  └────┬─────┘ └────┬─────┘ └────────┬─────────┘ │
│       │             │                │           │
│       └─────────────┴────────────────┘           │
│                     │ invoke() / listen()        │
└─────────────────────┼───────────────────────────┘
                      │ Tauri IPC
┌─────────────────────┼───────────────────────────┐
│                Rust 后端                          │
│  ┌──────────┐ ┌─────┴──────┐ ┌────────────────┐ │
│  │ Commands │ │ AudioEngine│ │   Database     │ │
│  │ (Tauri)  │ │ (Worker    │ │   (SQLite)     │ │
│  │          │ │  Thread)   │ │                │ │
│  └────┬─────┘ └─────┬──────┘ └───────┬────────┘ │
│       │             │                │           │
│       │     ┌───────┴────────┐       │           │
│       │     │  BASS FFI      │       │           │
│       │     │  (libloading)  │       │           │
│       │     └───────┬────────┘       │           │
│       │             │                │           │
│       ▼             ▼                ▼           │
│  ┌─────────┐  ┌──────────┐  ┌──────────────┐    │
│  │ Scanner │  │ bass.dll  │  │ music.db     │    │
│  │ (walkdir│  │ + 6 plugins│  │ (rusqlite    │    │
│  │  +lofty)│  │           │  │  bundled)    │    │
│  └─────────┘  └──────────┘  └──────────────┘    │
└──────────────────────────────────────────────────┘
```

### 音频线程模型

BASS API 不是线程安全的，所有 BASS 调用都在一个专用的 `audio-worker` 线程中执行：

```
Tauri Command (async)          Audio Worker Thread
     │                               │
     │── AudioCommand::Play ────────►│── BASS_StreamCreateFile
     │   (crossbeam-channel)         │── BASS_ChannelPlay
     │◄── Result<AudioStateDto> ─────│
     │                               │
     │                         ┌─────┴─────┐
     │                         │ recv_timeout(300ms)
     │                         │ → push_state()  ← 每 300ms 推送状态
     │                         │ → emit("audio://state")
     │                         └───────────┘
```

- Tauri command 通过 `crossbeam-channel` 向 worker 发送命令
- Worker 处理命令后通过 `oneshot` channel 返回结果
- Worker 每 300ms 通过 `recv_timeout` 检查状态并推送到前端
- 前端通过 `listen('audio://state')` 接收实时播放状态

### BASS DLL 加载流程

```
程序启动
  │
  ├─ resolve_audio_core_dir()
  │   1. Tauri resource_dir/audio_core     (打包后)
  │   2. EXE 同级 resources/audio_core      (绿色版)
  │   3. CARGO_MANIFEST_DIR/resources/...  (开发环境)
  │   4. vendor/audio_core/x64             (开发环境)
  │
  ├─ SetDllDirectoryW(audio_core_dir)      ← 添加 DLL 搜索路径
  │
  ├─ Library::new("bass.dll")              ← 动态加载 bass.dll
  │
  ├─ load_functions()                      ← 解析 21 个 BASS 函数符号
  │
  ├─ BASS_Init(-1, 44100, 0, NULL, NULL)  ← 初始化音频设备
  │
  └─ BASS_PluginLoad() × 6                ← 加载 6 个格式插件
      ├─ bassflac.dll
      ├─ bass_aac.dll
      ├─ bassopus.dll
      ├─ bassape.dll
      ├─ basswma.dll
      └─ bassalac.dll
```

### SQLite 数据库表结构

```sql
-- 歌曲表
CREATE TABLE songs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  path TEXT NOT NULL,           -- 文件完整路径
  path_key TEXT NOT NULL UNIQUE,-- 规范化路径（小写）用于去重
  file_name TEXT NOT NULL,
  title TEXT NOT NULL,
  artist TEXT, album TEXT, album_artist TEXT, genre TEXT,
  track_number INTEGER, disc_number INTEGER,
  duration_ms INTEGER DEFAULT 0,
  format TEXT, extension TEXT, codec_hint TEXT,
  size_bytes INTEGER, modified_time INTEGER,
  added_at INTEGER, updated_at INTEGER,
  last_played_at INTEGER, play_count INTEGER DEFAULT 0,
  playable INTEGER DEFAULT 1,   -- 是否可播放
  missing INTEGER DEFAULT 0,    -- 文件是否丢失
  last_error TEXT
);

-- 播放列表表
CREATE TABLE playlists (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  kind TEXT DEFAULT 'custom',   -- 'default' | 'custom'
  source_folder TEXT,
  sort_order INTEGER,
  created_at INTEGER, updated_at INTEGER
);

-- 播放列表-歌曲关联表
CREATE TABLE playlist_songs (
  playlist_id INTEGER NOT NULL,
  song_id INTEGER NOT NULL,
  sort_order INTEGER,
  added_at INTEGER,
  PRIMARY KEY (playlist_id, song_id)
);

-- 应用设置（键值对）
CREATE TABLE app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at INTEGER
);

-- 播放状态（单行）
CREATE TABLE playback_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  current_playlist_id INTEGER,
  current_song_id INTEGER,
  position_ms INTEGER DEFAULT 0,
  volume REAL DEFAULT 0.8,
  muted INTEGER DEFAULT 0,
  play_mode TEXT DEFAULT 'list_loop',
  updated_at INTEGER
);
```

---

## 四、Tauri Commands 一览

### 音频控制

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `audio_play` | songId: i64, path: String | AudioStateDto | 播放指定歌曲 |
| `audio_pause` | — | AudioStateDto | 暂停 |
| `audio_resume` | — | AudioStateDto | 恢复播放 |
| `audio_toggle_pause` | — | AudioStateDto | 切换暂停/播放 |
| `audio_stop` | — | AudioStateDto | 停止 |
| `audio_seek` | seconds: f64 | AudioStateDto | 跳转到指定秒数 |
| `audio_set_volume` | volume: f32 | AudioStateDto | 设置音量 (0.0–1.0) |
| `audio_get_state` | — | AudioStateDto | 获取当前状态 |
| `audio_set_play_mode` | mode: PlayMode | AudioStateDto | 设置播放模式 |
| `audio_get_diagnostics` | — | AudioDiagnosticsDto | 获取诊断信息 |

### 播放列表

| 命令 | 参数 | 返回 |
|------|------|------|
| `playlist_create` | name: String | PlaylistDto |
| `playlist_rename` | id: i64, name: String | PlaylistDto |
| `playlist_delete` | id: i64 | () |
| `playlist_clear` | id: i64 | () |
| `playlist_get_all` | — | Vec\<PlaylistDto\> |
| `playlist_get_songs` | playlistId: i64 | Vec\<SongDto\> |
| `playlist_song_reorder` | playlistId: i64, songIds: Vec\<i64\> | () |

### 歌库/扫描

| 命令 | 参数 | 返回 |
|------|------|------|
| `library_add_files` | paths: Vec\<String\>, playlistId: i64 | AddResultDto |
| `library_remove_song_from_playlist` | playlistId: i64, songId: i64 | () |
| `library_delete_song` | songId: i64 | () |
| `library_mark_song_unplayable` | songId: i64, error: String | () |
| `scan_start_folder` | path: String, playlistId: i64 | ScanTaskDto |
| `scan_cancel` | taskId: String | () |

### 设置

| 命令 | 参数 | 返回 |
|------|------|------|
| `settings_get` | — | HashMap\<String, String\> |
| `settings_update` | settings: HashMap\<String, String\> | HashMap\<String, String\> |
| `settings_get_playback_state` | — | PlaybackStateDto |
| `settings_save_playback_state` | playbackState: PlaybackStateDto | () |
| `settings_get_db_path` | — | String |

### 工具

| 命令 | 参数 | 返回 |
|------|------|------|
| `pick_music_files` | — | Vec\<String\> |
| `pick_folder` | — | Option\<String\> |
| `check_file_exists` | filePath: String | bool |
| `check_files_exist` | filePaths: Vec\<String\> | Vec\<(String, bool)\> |
| `show_in_folder` | filePath: String | () |
| `app_quit` | — | () |

### Tauri Events

| 事件名 | 触发时机 | Payload |
|--------|---------|---------|
| `audio://state` | 每 300ms 推送播放状态 | AudioStateDto |
| `audio://ended` | 歌曲自然播放结束 | { song_id, path, reason } |
| `audio://error` | 播放出错 | { song_id, path, code, message } |
| `scan://started` | 扫描开始 | { task_id, folder } |
| `scan://progress` | 扫描进度 | { task_id, scanned_files, added, ... } |
| `scan://finished` | 扫描完成 | { task_id, added, skipped, failed } |
| `scan://cancelled` | 扫描取消 | { task_id, added } |
| `library://changed` | 歌库变更 | {} |
| `playlist://changed` | 播放列表变更 | {} |

---

## 五、项目目录结构

```
music-player/
├── package.json                    # 前端依赖与脚本
├── vite.config.ts                  # Vite 配置
├── tsconfig.json                   # TypeScript 配置
├── index.html                      # HTML 入口
├── src/
│   ├── main.ts                     # Vue 应用入口
│   ├── App.vue                     # 根组件（关闭确认对话框）
│   ├── components/
│   │   ├── AppHeader.vue           # 标题栏（搜索/置顶/最小化/关闭）
│   │   ├── AppLayout.vue           # 主布局（拖放导入）
│   │   ├── Sidebar.vue             # 左侧播放列表栏（拖动排序）
│   │   ├── TrackList.vue           # 歌曲列表（虚拟滚动/右键菜单）
│   │   ├── PlayerBar.vue           # 底部播放控制栏（进度条/音量）
│   │   └── MiniPlayer.vue          # 迷你播放器
│   ├── stores/
│   │   ├── player.ts               # 播放状态（事件监听/invoke 控制）
│   │   ├── playlist.ts             # 播放列表（SQLite 读写）
│   │   └── app.ts                  # 应用配置（主题/窗口/设置持久化）
│   ├── types/
│   │   └── index.ts                # TypeScript 类型定义
│   ├── utils/
│   │   └── format.ts               # 格式化工具（时间/播放模式名）
│   └── styles/
│       └── main.css                # 全局样式 + CSS 变量
├── src-tauri/
│   ├── Cargo.toml                  # Rust 依赖
│   ├── tauri.conf.json             # Tauri 配置（窗口/打包/资源）
│   ├── build.rs                    # Tauri 构建脚本
│   ├── capabilities/
│   │   └── default.json            # 权限配置（窗口/事件/对话框）
│   ├── icons/                      # 应用图标
│   ├── resources/
│   │   └── audio_core/             # BASS DLL（打包进 EXE）
│   │       ├── bass.dll            # 核心引擎
│   │       ├── bassflac.dll        # FLAC 插件
│   │       ├── bass_aac.dll        # AAC/M4A 插件
│   │       ├── bassalac.dll        # ALAC 插件
│   │       ├── bassopus.dll        # Opus 插件
│   │       ├── bassape.dll         # APE 插件
│   │       └── basswma.dll         # WMA 插件
│   └── src/
│       ├── main.rs                 # 程序入口
│       ├── lib.rs                  # Tauri 应用配置（插件/命令注册/托盘）
│       ├── app_state.rs            # 全局状态（AudioEngine + Database）
│       ├── errors.rs               # 统一错误类型（AppError + AppErrorDto）
│       ├── audio/
│       │   ├── mod.rs              # 模块导出
│       │   ├── bass_ffi.rs         # BASS C FFI 类型与常量
│       │   ├── bass_loader.rs      # DLL 动态加载 + 符号解析
│       │   ├── audio_engine.rs     # AudioEngine（线程安全门面）
│       │   ├── audio_worker.rs     # 音频工作线程（BASS 调用/状态推送）
│       │   └── audio_state.rs      # 播放状态/模式/DTO 定义
│       ├── db/
│       │   ├── connection.rs       # SQLite 连接管理（WAL/路径解析）
│       │   ├── migrations.rs       # Schema 迁移（建表/默认数据）
│       │   ├── song_repo.rs        # 歌曲 CRUD
│       │   ├── playlist_repo.rs    # 播放列表 CRUD
│       │   └── settings_repo.rs    # 设置键值存储 + 播放状态
│       ├── library/
│       │   ├── scanner.rs          # 文件夹扫描（walkdir/去重/进度事件）
│       │   ├── metadata.rs         # 元数据读取（lofty）
│       │   └── supported_formats.rs# 支持的扩展名列表
│       ├── commands/
│       │   ├── audio_commands.rs   # 音频控制命令
│       │   ├── library_commands.rs # 歌库命令
│       │   ├── playlist_commands.rs# 播放列表命令
│       │   ├── scan_commands.rs    # 扫描命令
│       │   ├── settings_commands.rs# 设置命令
│       │   └── utility_commands.rs # 工具命令（文件选择/退出等）
│       └── settings/
│           └── settings.rs         # WindowStateDto 定义
└── README.md                       # 本文件
```

---

## 六、编译与构建

### 环境要求

- **Node.js** ≥ 18
- **Rust** (stable, edition 2021)
- **Windows 10/11** (x64)
- BASS DLL 文件放在 `src-tauri/resources/audio_core/` 目录下

### 开发模式

```bash
cd music-player
npm install
npm run tauri dev
```

开发模式下 Vite 提供热重载，Rust 代码修改后自动重新编译。

### 生产构建

```bash
cd music-player
npm install
npm run tauri build
```

构建产物：
- `src-tauri/target/release/music-player.exe` — 主程序
- `src-tauri/target/release/bundle/nsis/墨鱼听歌_1.0.0_x64-setup.exe` — NSIS 安装包

### Release 编译优化

`Cargo.toml` 中的 release profile 配置：

```toml
[profile.release]
panic = "abort"       # panic 时直接终止，不展开栈
codegen-units = 1     # 单编译单元，最大优化
lto = true            # 链接时优化
opt-level = "s"       # 优化体积
strip = true          # 剥离调试符号
```

### 清理编译缓存

```bash
# 删除 Rust 编译缓存（约 4 GB）
rm -rf src-tauri/target

# 删除前端依赖和构建产物
rm -rf node_modules dist
```

重新编译时 `npm install` + `npx tauri build` 会自动恢复。

---

## 七、BASS Audio Library 说明

### 授权提醒

BASS 是 un4seen.com 开发的第三方商业音频库。本项目仅用于个人本地非商业使用。如需商业分发，请自行确认并遵守 [BASS 官方授权](https://www.un4seen.com/)。

BASS DLL 必须来自官方包，不要伪造或生成 DLL。

### 封装的 BASS API

| 函数 | 用途 |
|------|------|
| BASS_Init | 初始化音频设备 |
| BASS_Free | 释放音频设备 |
| BASS_GetVersion | 获取 BASS 版本 |
| BASS_ErrorGetCode | 获取最后错误码 |
| BASS_PluginLoad | 加载格式插件 |
| BASS_PluginFree | 卸载格式插件 |
| BASS_StreamCreateFile | 从文件创建播放流 |
| BASS_StreamFree | 释放播放流 |
| BASS_ChannelPlay | 播放通道 |
| BASS_ChannelPause | 暂停通道 |
| BASS_ChannelStop | 停止通道 |
| BASS_ChannelSetPosition | 设置播放位置（seek） |
| BASS_ChannelGetPosition | 获取播放位置 |
| BASS_ChannelGetLength | 获取总时长 |
| BASS_ChannelBytes2Seconds | 字节位置转秒数 |
| BASS_ChannelSeconds2Bytes | 秒数转字节位置 |
| BASS_ChannelSetAttribute | 设置通道属性（音量等） |
| BASS_ChannelGetAttribute | 获取通道属性 |
| BASS_ChannelIsActive | 获取通道活跃状态 |
| BASS_ChannelSetSync | 设置同步回调（播放结束检测） |
| BASS_SetConfig | 全局配置（可选） |

### 错误处理

BASS 错误码转换为中文友好提示：

| 错误码 | 含义 |
|--------|------|
| BASS_ERROR_FILEOPEN | 无法打开文件 |
| BASS_ERROR_FORMAT | 文件格式不支持 |
| BASS_ERROR_CODEC | 编解码器不支持，可能需要安装插件 |
| BASS_ERROR_FILEFORM | 文件格式无法识别或编码不支持 |
| BASS_ERROR_DECODE | 解码失败，文件可能已损坏 |
| BASS_ERROR_NOCHAN | 无法创建播放通道 |

播放失败时不崩溃、不 panic，返回错误并自动跳下一首。

---

## 八、运行时目录结构

### 绿色版（EXE 直接运行）

```
墨鱼听歌/
├── 墨鱼听歌.exe              ← 主程序
├── resources/
│   └── audio_core/           ← BASS DLL（7 个文件）
│       ├── bass.dll
│       ├── bassflac.dll
│       ├── bass_aac.dll
│       ├── bassalac.dll
│       ├── bassopus.dll
│       ├── bassape.dll
│       └── basswma.dll
├── data/                     ← SQLite 数据库（自动创建）
│   └── music.db
└── logs/                     ← 日志文件（自动创建）
    └── app.log
```

### 数据库路径解析优先级

1. EXE 同级 `data/music.db`（绿色版首选）
2. Windows AppData 目录（EXE 目录不可写时回退）

---

## 九、权限配置

`capabilities/default.json` 定义了应用的安全权限：

```json
{
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-close",
    "core:window:allow-minimize",
    "core:window:allow-hide",
    "core:window:allow-show",
    "core:window:allow-set-size",
    "core:window:allow-set-position",
    "core:window:allow-set-always-on-top",
    "core:window:allow-start-dragging",
    "core:window:allow-outer-size",
    "core:window:allow-outer-position",
    "core:window:allow-set-maximizable",
    "core:window:allow-unminimize",
    "core:window:allow-set-focus",
    "core:tray:default",
    "core:event:default",
    "core:event:allow-listen",
    "core:event:allow-emit",
    "dialog:default"
  ]
}
```

不需要 HTTP 请求、远程接口、自动更新、账号登录等网络权限。

---

## 十、特点与设计决策

### 为什么不用 mpv / rodio / symphonia / FFmpeg？

| 方案 | 问题 |
|------|------|
| mpv | 需要用户安装 mpv 并配置 PATH，不是绿色版 |
| rodio + symphonia | seek 不稳定，某些 M4A 会 panic 导致主程序崩溃 |
| FFmpeg 进程 | 每次 seek 启动新进程，延迟 1-2 秒，连续拖动卡死 |
| HTML5 Audio | 格式支持有限（FLAC/WMA/APE 无法播放） |

### BASS 方案优势

- DLL 内置，不需要安装任何外部软件
- seek 通过 `BASS_ChannelSetPosition` 即时生效，无延迟
- M4A 同时支持 AAC 和 ALAC 两种编码
- 播放失败返回错误而非 panic，不会杀死主程序
- 体积小，7 个 DLL 总共不到 1 MB

### 线程安全

- 所有 BASS 调用集中在 `audio-worker` 线程
- Tauri command 通过 channel 发送命令，不直接调用 BASS
- SQLite 连接使用 `parking_lot::Mutex` 保护
- 状态推送通过 Tauri event 系统，线程安全

### 进度条拖动设计

- `mousedown`：设 `isSeeking = true`，停止跟随后端 position
- `mousemove`：只更新前端 UI 显示时间，不调用后端
- `mouseup`：一次性调用 `audio_seek(seconds)`，完成后恢复跟随
- 后端 300ms 推送的 position 在 `isSeeking` 时不覆盖 UI

---

## 十一、常见问题

### Q: 双击 EXE 闪退？

检查 `resources/audio_core/` 目录是否存在且包含 `bass.dll`。DLL 缺失时程序会报错退出。

### Q: 某些 M4A 无法播放？

M4A 是容器格式，内部可能是 AAC 或 ALAC。确保 `bass_aac.dll` 和 `bassalac.dll` 都存在。无法播放的歌曲会被标记，自动跳到下一首。

### Q: WMA 播放失败？

WMA 依赖系统组件，某些精简版 Windows 可能缺少 WMA 解码器。插件加载失败不会崩溃，只会禁用对应格式。

### Q: 数据库在哪里？

绿色版：EXE 同级 `data/music.db`。安装版：`%APPDATA%/com.musicplayer.app/music.db`。

### Q: 如何查看日志？

日志文件在 `logs/app.log`（与 EXE 同级或 AppData 目录下），按天滚动。

### Q: 如何重新编译？

```bash
cd music-player
npm install
npm run tauri build
```

产物在 `src-tauri/target/release/music-player.exe`。
# 轻听音乐 - 本地音乐播放器

一个轻量级、简洁、美观的 Windows 本地音乐播放器，灵感来自经典「千千静听」。

## 特性

- 纯本地运行，无需联网
- 支持 MP3 / FLAC / WAV / OGG / AAC / M4A / WMA / APE / OPUS 格式
- 自由创建、管理播放列表
- 拖拽添加文件和文件夹
- 文件夹拖入侧栏自动创建播放列表
- 深色 / 浅色主题切换
- 迷你模式
- 窗口置顶
- 搜索歌曲
- 播放模式：顺序 / 单曲循环 / 列表循环 / 随机
- 自动记住上次播放状态
- 虚拟列表优化，万首歌曲也流畅
- 最终打包为小体积 Windows EXE

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面容器 | Tauri 2.x |
| 前端 | Vue 3 + TypeScript + Vite + Pinia |
| 后端 | Rust (lofty 音频元数据, walkdir 文件扫描) |
| 音频播放 | HTML5 Audio API + 格式 fallback |

---

## 环境准备

### 1. 安装 Node.js

推荐 Node.js 18+，从 https://nodejs.org 下载 LTS 版本安装。

### 2. 安装 Rust

从 https://rustup.rs 下载 `rustup-init.exe` 安装 Rust 工具链：

```bash
# 安装时选择默认 (stable-x86_64-pc-windows-msvc)
# 安装完成后验证：
rustc --version
cargo --version
```

### 3. 安装 Microsoft Visual Studio C++ Build Tools

Tauri 2 在 Windows 上需要 MSVC 编译器和 Windows SDK：

1. 访问 https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. 下载并运行 **Visual Studio Build Tools 2022**
3. 在安装程序中勾选：
   - **使用 C++ 的桌面开发**（Desktop development with C++）
   - 确保包含：**Windows 11 SDK** 和 **MSVC v143 生成工具**
4. 安装完成后重启电脑

### 4. WebView2 运行时

Windows 10/11 通常已预装 WebView2。如果没有，从以下地址安装：
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

---

## 开发运行

```bash
# 进入项目目录
cd music-player

# 安装前端依赖
npm install

# 启动开发服务器（首次运行会编译 Rust 后端，耗时较长）
npm run tauri dev
```

首次运行 `tauri dev` 时会自动下载和编译 Rust 依赖，可能需要 5-10 分钟。后续运行会快很多。

---

## 打包发布

### 生成安装包（NSIS）

```bash
npm run tauri build
```

打包完成后，安装包位于：
```
src-tauri/target/release/bundle/nsis/轻听音乐_1.0.0_x64-setup.exe
```

### 生成 MSI 安装包

```
src-tauri/target/release/bundle/msi/轻听音乐_1.0.0_x64_en-US.msi
```

### 便携版 EXE

编译后的主程序可直接运行，无需安装：
```
src-tauri/target/release/music-player.exe
```

将 `music-player.exe` 复制到任意位置即可作为便携版使用。

---

## 项目结构

```
music-player/
├─ package.json              # 前端依赖和脚本
├─ index.html                # HTML 入口
├─ vite.config.ts            # Vite 配置
├─ tsconfig.json             # TypeScript 配置
├─ src/                      # 前端源码
│   ├─ main.ts               # Vue 入口
│   ├─ App.vue               # 根组件（初始化 + 状态管理）
│   ├─ types/
│   │   └─ index.ts          # TypeScript 类型定义
│   ├─ core/
│   │   └─ audio-engine.ts   # HTML5 Audio 播放引擎
│   ├─ stores/
│   │   ├─ playlist.ts       # 播放列表状态管理
│   │   ├─ player.ts         # 播放器状态管理
│   │   └─ app.ts            # 应用配置状态管理
│   ├─ components/
│   │   ├─ AppLayout.vue     # 主布局
│   │   ├─ AppHeader.vue     # 标题栏（搜索 + 窗口控制）
│   │   ├─ Sidebar.vue       # 播放列表侧栏
│   │   ├─ TrackList.vue     # 歌曲列表（虚拟滚动）
│   │   ├─ PlayerBar.vue     # 底部播放控制栏
│   │   └─ MiniPlayer.vue    # 迷你模式
│   ├─ utils/
│   │   └─ format.ts         # 格式化工具函数
│   └─ styles/
│       └─ main.css          # 全局样式 + 主题变量
├─ src-tauri/                # Tauri/Rust 后端
│   ├─ Cargo.toml            # Rust 依赖
│   ├─ tauri.conf.json       # Tauri 配置
│   ├─ build.rs              # Tauri 构建脚本
│   ├─ capabilities/
│   │   └─ default.json      # 权限配置
│   └─ src/
│       ├─ main.rs           # 程序入口
│       ├─ lib.rs            # 命令注册
│       ├─ commands.rs       # Tauri 命令实现
│       └─ models.rs         # 数据模型
└─ README.md                 # 本文件
```

---

## 使用说明

### 添加音乐

- 点击标题栏「添加音乐」按钮选择文件
- 点击标题栏「添加文件夹」按钮扫描整个文件夹
- 直接将文件或文件夹拖入主界面

### 播放列表管理

- 点击侧栏「+」新建播放列表
- 将文件夹拖到侧栏空白处，自动以文件夹名创建播放列表
- 将文件拖到特定播放列表上，添加到该列表
- 右键歌曲可播放、在文件夹中显示、移除

### 播放控制

- 双击歌曲立即播放
- 底部控制栏：播放/暂停、上一首/下一首
- 拖动进度条跳转
- 拖动音量条调节音量
- 点击播放模式图标切换模式

### 搜索

- 在顶部搜索框输入关键词，实时过滤当前列表

### 主题切换

- 深色主题和浅色主题自动跟随系统（也可手动切换）

### 迷你模式

- 点击标题栏迷你模式按钮，缩小为精简控制条
- 点击展开按钮恢复完整界面

### 窗口置顶

- 点击标题栏置顶按钮，窗口始终显示在最前

---

## 音频格式支持

| 格式 | 支持方式 | 说明 |
|------|---------|------|
| MP3 | HTML5 Audio 原生 | 最通用格式 |
| WAV | HTML5 Audio 原生 | 无损未压缩 |
| OGG | HTML5 Audio 原生 | 开源格式 |
| AAC | HTML5 Audio 原生 | 高质量压缩 |
| M4A | HTML5 Audio 原生 | Apple 格式 |
| OPUS | HTML5 Audio 原生 | 新一代编码 |
| FLAC | Rust 后端 fallback | 无损压缩 |
| WMA | Rust 后端 fallback | Windows 格式 |
| APE | Rust 后端 fallback | 无损格式 |

对于 HTML5 Audio 原生不支持的格式（FLAC/WMA/APE），程序通过 Rust 后端将文件读取为 data URL 传递给 Audio 引擎播放。

---

## 后续扩展方向

- **FFmpeg 解码集成**：通过 Rust FFmpeg 绑定实现全格式硬件解码
- **均衡器**：Web Audio API 实现图形均衡器
- **快捷键**：全局键盘快捷键控制播放
- **系统托盘**：最小化到系统托盘后台播放
- **歌词显示**：本地 LRC 歌词文件解析
- **音频可视化**：Canvas 频谱动画

---

## 常见问题

### Q: 首次 `npm run tauri dev` 编译很慢？
A: 正常现象。首次需要下载和编译所有 Rust 依赖（约 200+ 个 crate），通常需要 5-15 分钟。后续编译会使用缓存，只需几秒。

### Q: 提示找不到 MSVC 编译器？
A: 需要安装 Visual Studio Build Tools 2022，勾选「使用 C++ 的桌面开发」工作负载。

### Q: 某些 FLAC 文件无法播放？
A: 当前通过 base64 data URL 方式播放，对大文件（>50MB）可能有延迟。后续可通过 FFmpeg 或 asset protocol 优化。

### Q: 如何减小打包体积？
A: `Cargo.toml` 中已配置 release profile 优化（LTO + strip + opt-level=s）。最终 EXE 约 5-8MB。

---

## 许可证

MIT License - 仅供个人使用
