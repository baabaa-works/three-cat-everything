# three-cat-everything — 執行計畫

規格見 [SPEC.md](SPEC.md)。本檔給執行階段照著做，每個里程碑都有驗證方式，驗證過了再往下。

## 環境現況（2026-11-10 檢查）

| 項目 | 狀態 |
|---|---|
| Node v24 / npm / pnpm / Python 3.14 / git / gh / winget | 已安裝 |
| WebView2 Runtime | 已安裝（157） |
| gh 帳號 | 已登入，對 `baabaa-works` 有 admin 權限 |
| Rust（rustup / cargo） | **未安裝** |
| Visual Studio C++ Build Tools（MSVC linker） | **未安裝** |
| ffmpeg / yt-dlp | **未安裝**（改用 pip 套件） |

## M0 — 環境準備

1. `winget install Rustlang.Rustup`，確認 `stable-x86_64-pc-windows-msvc`。
2. `winget install Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"`
   - 約 6–8GB、10–20 分鐘，**會跳 UAC，需要使用者按一次「是」**。用背景執行，同時先做 M1。
3. 在專案內建 Python venv：`pip install yt-dlp imageio-ffmpeg`。
4. `git init`、建立 `.gitignore`、`gh repo create baabaa-works/three-cat-everything --private --source . --remote origin`。

驗證：`cargo --version`、`cl.exe` 找得到（vswhere）、venv 內 `python -c "import imageio_ffmpeg; print(imageio_ffmpeg.get_ffmpeg_exe())"`。

## M1 — 素材管線（不依賴 Rust，可與 M0 平行）

腳本放 `tools/make_asset.py`，原始檔放 `raw/`（加進 `.gitignore`），可重跑。

1. yt-dlp 下載綠幕影片到 `raw/source.mp4`；用 ffprobe 資訊確認解析度、長度、有音軌。
2. 抽一格看：實際綠色色碼（取角落像素）、底部是否有燒進去的字幕／浮水印、貓的外框。
3. ffmpeg 濾鏡鏈：`crop`（貓外框，留少量邊）→ `chromakey`（色碼取樣值，similarity/blend 從 0.12/0.06 起調）→ `despill=green` → `format=yuva420p`。
4. 編碼：`libvpx-vp9 -pix_fmt yuva420p -b:v 0 -crf 32 -auto-alt-ref 0` ＋ `libopus 96k`，輸出 `src/assets/cats.webm`。
5. 抽 3 格疊在深色、淺色、紅色（看盤軟體常見色）背景上輸出 PNG，檢查綠邊與破洞；不乾淨就調 similarity／blend，或加 1px 的 alpha 侵蝕。
6. 取一格去背圖產生 `app-icon.png`（1024×1024），之後餵給 `pnpm tauri icon`。

驗證：三張合成預覽圖給使用者看；檔案 ≤ 3MB；用 ffprobe 確認 `alpha_mode=1`。

風險與備案：
- imageio-ffmpeg 內建的 ffmpeg 若沒有 libvpx-vp9 → 改 `winget install Gyan.FFmpeg`。
- TikTok 擋 yt-dlp → 用內建瀏覽器開影片頁，從 network 請求取影片網址；再不行就換備案來源。
- 綠幕品質差 → 換 @davi.editor05 的 576×576 版本。

## M2 — 專案骨架與單一視窗

1. `pnpm create tauri-app`（vanilla-ts 樣板）建在專案根目錄，app identifier `works.baabaa.three-cat-everything`。
2. `tauri.conf.json`：不宣告預設視窗（視窗全部由 Rust 依狀態建立）；bundle 目標只留 `nsis`。
3. Rust 端 `create_cat_window(id, state)`：`transparent(true) / decorations(false) / always_on_top(true) / skip_taskbar(true) / shadow(false) / resizable(false)`。
4. 前端：全螢幕 `<video autoplay loop muted playsinline>`，`html, body { background: transparent }`。

驗證：`pnpm tauri dev` 啟動後截圖，貓疊在桌面上、背景透明、持續循環。**這是全案最大的技術風險點（WebView2 透明視窗 + VP9 alpha），先過這關再做其他功能。**

備案：若 WebView2 播不出 alpha → 改成 `<canvas>` 即時去背（WebGL shader 對綠幕 mp4 做 chroma key），素材改用原始綠幕 mp4。

## M3 — 互動

1. 拖曳與單擊：`pointerdown` 記錄座標，`pointermove` 超過 4px 才呼叫 `startDragging()`；沒拖曳就在 `pointerup` 當成單擊 → 切換 `video.muted`。
2. 滾輪縮放：改變視窗大小並維持影片長寬比，以視窗中心為基準，限制 120–900px。
3. 音樂互斥：開始播放時 `emit("music-owner", id)`，其他視窗收到就靜音。
4. 鎖定位置：鎖定時忽略拖曳與滾輪。

驗證：實機操作，確認單擊不會誤觸拖曳、拖曳不會誤觸音樂。

## M4 — 選單、系統匣、多組

1. 右鍵：前端攔 `contextmenu`，Rust 端用 `Menu` 的 `popup` 顯示原生選單（項目見 SPEC §4），勾選狀態跟著該組狀態。
2. 系統匣：`TrayIconBuilder` ＋ 選單。
3. 多組：`add_group` / `remove_group`，視窗 label `cat-{id}`，上限 5；新的一組出現在目前那組旁邊偏移 40px。
4. 滑鼠穿透：`set_ignore_cursor_events(true)`；系統匣「解除全部滑鼠穿透」負責復原。
5. 外掛：`tauri-plugin-autostart`（開機啟動）、`tauri-plugin-single-instance`、`tauri-plugin-opener`（關於裡的連結）。
6. 「關於」：小型一般視窗（有邊框），顯示版本、`Original animation by @meowtakeover`、repo 連結。

驗證：開到 5 組確認第 6 組被擋；穿透開啟後能點到下方視窗，並可從系統匣解除。

## M5 — 狀態保存

1. `tauri-plugin-store`，結構照 SPEC §5；Rust 端持有單一 `AppState`（`Mutex`），前端只送事件。
2. 視窗 `Moved` / `Resized` 事件 debounce 500ms 後寫入。
3. 啟動還原，並用 `available_monitors()` 檢查位置是否仍在螢幕內。

驗證：擺好 3 組 → 結束 → 重開，位置、大小、透明度、鎖定都還原。

## M6 — 打包與交付

1. `pnpm tauri icon app-icon.png`。
2. `pnpm tauri build` → NSIS 安裝檔；實際安裝一次、從開始選單啟動、再解除安裝。
3. README（繁中為主，附英文簡介）：截圖／GIF、功能、操作說明、作者標註、SmartScreen 警告說明。
4. commit、push 到 private repo。
5. 走完 SPEC §6 的完成清單，截圖回報。**改 public 與發 Release 要等使用者明確同意。**

## 需要使用者介入的地方

- M0 的 Build Tools 安裝會跳一次 UAC。
- M6 之後：是否改 public、是否發 Release。

## 執行時自行決定、最後在總結列出的假設

- 預設大小、縮放範圍、選單文字用語（繁中）。
- chroma key 參數與裁切範圍。
- 圖示用哪一格。
- 選單只提供繁體中文，不做多語系。
