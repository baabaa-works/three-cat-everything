# 下載漏斗 SPEC：從 Threads 點連結到小貓出現在桌面

給執行 session（Opus 5.5）照做到完成。決定都已和使用者定案（2026-11-10 grill 討論），不要重新詢問。

## 目標

看了 Threads 宣傳片的台股散戶，點連結後能用最少步驟把小貓裝起來。這一波只改 README 和 release，不改 app。

入口固定是 repo 首頁 `github.com/baabaa-works/three-cat-everything`（已印在影片片尾與貼文草稿裡，不能換）。

## 現況（2026-11-10 查到）

- README 第一屏：標題 → 一句話 → 靜態預覽圖 `docs/preview.png` → 「下載」段落要再點進 Releases 頁找 exe。共 3 次點擊才開始下載。
- Release `v0.1.0` 只有 `three-cat-everything_0.1.0_x64-setup.exe`（4,022,221 bytes，sha256 `25f869a7220d419e852188ef2bbce89e193189bb62b4fd9bf136648b6c54fe08`）。
- 安裝檔沒簽章：SmartScreen 會跳警告；開了「智慧型應用程式控制」的電腦裝不了。README 只有文字說明。
- 宣傳片成品在主 checkout 的 `docs/promo/out/three-cats-promo.mp4`（gitignored，用絕對路徑 `C:\Users\User\Projects\SIDE-PJ\three-cat-everything\docs\promo\out\three-cats-promo.mp4` 讀）。

## 已定案的決定

| # | 項目 | 決定 |
|---|---|---|
| 1 | 手機使用者 | README 第一屏放一行提示：用電腦開這頁，或先按 ⭐ 收藏。不做寄連結功能。 |
| 2 | 落地頁 | 只改 README 第一屏，不另做 GitHub Pages。 |
| 3 | 下載點擊數 | 第一屏放大的「下載 Windows 版」按鈕，一點直接下載安裝檔。Releases 連結留在下面當備用。 |
| 4 | SmartScreen | 不買簽章。下載按鈕下方放兩張實際截圖（「其他資訊」→「仍要執行」）並一句解釋為何會跳。 |
| 5 | 成功的定義與量測 | 終點是「小貓出現在桌面」。只用 GitHub release 下載數與 repo traffic 量，app 不加回報。 |
| 6 | 固定下載連結 | 每個 release 多傳一份固定檔名 `three-cat-everything-setup.exe`，按鈕連 `https://github.com/baabaa-works/three-cat-everything/releases/latest/download/three-cat-everything-setup.exe`。帶版本號的原檔保留。v0.1.0 現在就補。 |
| 7 | README 第一屏順序 | 見下節。 |
| 8 | SmartScreen 截圖來源 | 在這台電腦實際下載、執行一次取得。截不到或點不了就請使用者自己截兩張。 |
| 9 | 智慧型應用程式控制 | README 只寫一句「看到『智慧型應用程式控制已封鎖』的話目前裝不了」並請到 issue 留言，不寫關閉教學。關閉教學由使用者自己貼在 Threads 留言區，本 SPEC 附草稿。上架 Microsoft Store 列為後續工作，這波不做。 |
| 10 | App 本身 | 不改（不加首次提示、不改預設位置）。 |
| 11 | Threads 連結露出 | 主貼文、自己貼文下第一則回覆、個人檔案簡介三處都放連結。由使用者自己做，執行 session 只在回報裡提醒。 |
| 12 | 上線方式 | 從新分支做，push 後開 PR，使用者自己按合併。release 的固定檔名副本可直接上傳。 |
| 13 | 驗收 | 見「驗收」一節；Threads 內建瀏覽器實測由使用者發文前自己用手機點一次。 |

## README 第一屏（由上到下）

1. 標題與一句話（沿用現有文案）。
2. 動圖：從宣傳片剪 3–4 秒「小貓在 K 線上跳」（建議 14–17.5 秒段，兩組都在畫面中），只取錄影框不含字幕，轉成寬約 480px 的 GIF 或 WebP 動圖，檔案 3MB 內，放 `docs/`。取代第一屏的 `docs/preview.png`（原圖保留在下方或刪除皆可）。
3. 下載按鈕：GitHub README 不能放真按鈕，用一張按鈕樣式的圖片或 shields.io 徽章包連結，文字「下載 Windows 版」。旁邊或下方一行小字：「免費・約 4MB・Windows 10 / 11（64 位元）」。
4. 手機提示一行：「📱 用手機看到？用電腦開這頁，或先按右上角 ⭐ 收藏」。
5. 「安裝三步驟」：① 點上面按鈕下載 ② 雙擊安裝檔，若出現「Windows 已保護您的電腦」，點「其他資訊」→「仍要執行」（兩張截圖）③ 裝好後小貓出現在螢幕右下角。附一句為何會跳警告（沒有付費程式碼簽章）。
6. 一句智慧型應用程式控制說明（決定 9）。
7. 「裝好之後」三行：小貓在右下角、左鍵拖曳移動、右鍵叫出選單（再來一組、滑鼠穿透）。預設靜音，點一下播音樂。

以下維持現有內容、往下移：完整「怎麼玩」表格、透明度示意圖、開發、重做素材、Credits、English。

文案用繁體中文，語氣與現有 README 一致。不要碰「反指標」「禁貓」「看了就綠」這類梗，不提其他創作者名字以外的 credits 變動。

## 步驟

1. 從目前分支開新分支 `download-funnel`（目前分支 `promo-video` 已含宣傳片專案與本 SPEC，一起進 PR 沒問題）。
2. 上傳固定檔名副本到 v0.1.0：下載原檔（`gh release download v0.1.0`），驗證 sha256 與上面一致，複製成 `three-cat-everything-setup.exe` 後 `gh release upload v0.1.0`。確認未登入狀態下 `curl -sIL <latest/download 連結>` 最後是 200、大小 4,022,221。
3. 做動圖（ffmpeg，裁出錄影框區域；成片錄影框位於 x 20–1060、y 340–1640）。
4. 取得 SmartScreen 截圖：用瀏覽器（可用內建瀏覽器或 Chrome）開 latest/download 連結下載，從檔案總管執行，截 SmartScreen 兩個畫面。computer-use 可能看得到但點不了系統對話框；點不了就用 SendMessage 問一次請使用者代點或代截，其餘先做。截圖只框住對話框，不含桌面其他內容與個資，存 `docs/install-1.png`、`docs/install-2.png`。走完安裝後確認小貓出現，截一張存證（不進 repo）。
   - 這會在這台電腦重裝同一版，使用者既有的小貓設定（`%APPDATA%\works.baabaa.three-cat-everything\state.json`）先備份，結束後若被改動就還原。
5. 改 README 第一屏（上節）。順便把 README「下載」段落的版本號檔名說明改成固定連結。
6. 本機用內建瀏覽器預覽 README（GitHub 上的分支頁面即可，push 後看 `https://github.com/baabaa-works/three-cat-everything/tree/download-funnel`），手機寬度 375px 與桌面寬度各截一張。
7. 更新 release 說明（`gh release edit v0.1.0 --notes-file`）：開頭加固定檔名的直接下載說明，其餘保留。
8. commit、push `download-funnel`、開 PR（不合併）。PR 描述列出改了什麼與驗收截圖。
9. 回報給使用者：PR 連結、驗收截圖、Threads 提醒（決定 11）、下方的留言區草稿，並請使用者發文前用手機在 Threads 點一次連結。

## 驗收

PR 合併前（分支頁）與合併後（repo 首頁）各一次：

1. 手機寬度（375×812）開頁面：第一屏不捲動就看得到動圖、下載按鈕、手機提示。截圖。
2. 桌面寬度：點下載按鈕直接開始下載，下載的檔案大小 4,022,221、sha256 與原檔一致。
3. 執行下載的檔案：走完 SmartScreen 與安裝，小貓出現。
4. 未登入狀態下固定連結可下載（curl 驗證）。
5. 動圖在 GitHub 上會動、3MB 內；截圖不含個資。
6. 截圖與結果用 SendUserFile 傳給使用者。

合併後那一次驗收在使用者按下合併之後才能做；PR 開好時先做合併前那次，合併後的那次留給使用者合併後通知再做，或由使用者自己看首頁。

## 不做的事

- 不改 app、不改宣傳片、不發任何貼文、不登入 Threads、不合併 PR。
- 不教人關閉智慧型應用程式控制（README 內）。
- 不買簽章、不上 Store、不加 app 回報。

## Threads 留言區草稿：智慧型應用程式控制（使用者自行決定是否貼）

> 有人安裝時看到「智慧型應用程式控制已封鎖此應用程式」嗎？
> 這是 Windows 11 新電腦才有的安全功能，會擋掉沒有付費簽章的小程式，小貓就是其中之一。
>
> 想裝的話可以到「設定 → 隱私權與安全性 → Windows 安全性 → 應用程式與瀏覽器控制 → 智慧型應用程式控制設定」把它改成「關閉」。
> ⚠️ 注意：關掉之後通常沒辦法再打開，除非重灌 Windows。不確定的話就先不要關，我之後會想辦法上架到 Microsoft Store。

執行 session 要用一台 Windows 11 確認這條設定路徑的名稱是否正確（這台電腦有 Windows 安全性可以看，只看不改），不對就修正草稿文字後再回報。

## 後續（這波不做）

- 上架 Microsoft Store（MSIX，Store 代簽），可同時解決 SmartScreen 與智慧型應用程式控制。
- 簽章憑證選項：Certum 開源憑證、SignPath Foundation。費用與資格需屆時再查。
