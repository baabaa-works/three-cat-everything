# 宣傳影片 SPEC

給執行 session（Opus 5.5）照著做到完成。規劃討論已結束，下列決定不要重新詢問使用者。

## 目標

一支 Threads 用的直式短片，讓看盤的台股散戶看完去下載 three-cat-everything。

- 唯一 CTA：下載。連結 `github.com/baabaa-works/three-cat-everything`（v0.1.0 release 已上架）。
- 規格：1080×1920、30fps、mp4（H.264 + AAC）、長度 15–25 秒。
- 前 2 秒內畫面上要有 K 線；小貓在第 3 秒左右登場。
- 只做這一支，不出橫式版。

## 已定案的決定

| 項目 | 決定 |
|---|---|
| 背景 | 使用者會把 TradingView 的國巨（2327）畫面開在副顯示器。只當背景借景，不必等實盤，不後製改任何數字或 K 線。 |
| 操作與錄影 | 全部由執行 session 自己做：用 computer-use 操作小貓、用 ffmpeg 錄副顯示器。使用者只負責開好 TradingView。 |
| 小貓來源 | 真的 app 實錄（`src-tauri/target/release/three-cat-everything.exe`）。備案見「風險」。 |
| 游標 | 錄影時隱藏。 |
| 示範功能 | 拖到 K 線上、滾輪放大、右鍵「再來一組」變兩組。滑鼠穿透只用字幕帶過。透明度、鎖定、開機自啟不提。 |
| 版面 | 錄影裁成 4:5 區塊（1080×1350）放在直式畫面中段，上下留給字幕與片尾資訊。 |
| 聲音 | 用小貓原本的迷因音樂（取自 `src/assets/cats.webm` 的音軌，11.81 秒，循環鋪滿）。不加旁白。 |
| 文案調性 | 有梗，但不碰「反指標」「禁貓」「看了就綠」這類梗。 |
| 稱呼 | 影片與貼文用「三隻小貓」加「漲停」；產品名 three-cat-everything 不動。 |
| 其他創作者 | 哲哲、巴逆逆等人的名字、頭像、貼文截圖都不進影片。 |
| 免責 | 片尾卡加「非投資建議」小字。 |
| 驗收流程 | 跳過 hyperframes 的預覽核准，直接 render，自行抽格檢查後交成品。 |

## 字幕文案（使用者定稿，逐字照用）

| # | 時間（約） | 畫面 | 字幕 |
|---|---|---|---|
| 1 | 0–3s | 只有 K 線 | 台股開盤，國巨不往上動 |
| 2 | 3–6s | 小貓出現在 K 線上 | 沒關係，派出小貓！ |
| 3a | 6–9s | 小貓被拖動、放大 | 拖到 K 線上・滾輪放大 |
| 3b | 9–13s | 右鍵選單 → 第二組出現 | 一組不夠看？再來一組🔥 |
| 4 | 13–16s | 兩組在 K 線上跳 | 開滑鼠穿透，不擋你掛漲停價 |
| 5 | 16–20s | 同上，可略推近 | 一張傳三代，人走貓還在 |
| 6 | 20–23s | 片尾卡 | 三隻小貓・免費下載・Windows |

片尾卡另含：

- `github.com/baabaa-works/three-cat-everything`
- `Original animation by @meowtakeover · Green-screen by @igreenscreenthings`
- 小字「非投資建議」

時間點可微調，文字不可改。

## 工具

- 錄影：ffmpeg 的 `ddagrab`，`draw_mouse=0`。
- 操作 app：computer-use MCP（先 `request_access`，再 `switch_display` 到副顯示器）。
- 合成：`hyperframes` / `hyperframes-cli` / `hyperframes-registry` skill，開工前先讀它們的 SKILL.md。直式安全區照 `references/types/vertical.md`。
- 素材處理：`media-use` skill 或直接用 ffmpeg。

## 環境事實（2026-11-10 查到）

- Node v24.18.0、Python 3.14.4 已安裝；hyperframes 的 Chrome headless shell 已在快取。
- ffmpeg、ffprobe 都不在 PATH。專案 `.venv` 裡有 imageio-ffmpeg 9.0.2（支援 `ddagrab`），但沒有 ffprobe。
- 使用者已同意用 `winget install Gyan.FFmpeg` 裝完整版（含 ffprobe）。裝完若 hyperframes 找不到，設 `HYPERFRAMES_FFMPEG_PATH` / `HYPERFRAMES_FFPROBE_PATH`。
- 顯示器：主螢幕 `DISPLAY5` 邏輯 1707×960 於 (0,0)；副螢幕 `DISPLAY1` 邏輯 1536×960 於 (2048,0)。實體像素未量，`ddagrab` 的 `output_idx` 要實測。
- app 是透明、無邊框、always-on-top 的視窗；預設靜音。

## 步驟

1. 安裝 ffmpeg，確認 `ffmpeg -version`、`ffprobe -version`。
2. **先做 3 秒試錄**：啟動 release 版 exe，用 `ddagrab` 錄副顯示器，抽一格確認透明小貓有被錄到、游標沒出現。錄不到就走備案。
3. 確認使用者已把 TradingView 開在副顯示器、國巨 K 線主圖在螢幕中央。沒開就請使用者開（這是唯一允許停下來的地方）。
4. 正式錄影，一條長 take 即可，之後在時間軸上取段：
   - 先錄 4 秒以上純 K 線（小貓隱藏或尚未啟動）。
   - 小貓出現，拖到 K 線中央。
   - 滾輪放大。
   - 右鍵 →「再來一組」，把第二組拖到旁邊。
   - 兩組一起跳，至少再錄 10 秒。
   - 每個動作之間停 1–2 秒，方便剪。
5. 用 hyperframes 建 1080×1920 專案於 `promo/`：錄影裁 4:5 置中、字幕、片尾卡、音軌循環。字幕和片尾卡的樣式優先找 registry 現成元件。
6. `lint` / `check` 通過後 render，輸出到 `docs/promo/out/three-cats-promo.mp4`。
7. 驗收（見下），不過就修到過。
8. 把 Threads 貼文草稿（見下）連同成品路徑回報給使用者。不要代發任何貼文。

原始錄影放 `raw/promo/`（`raw/` 已被 gitignore）。不要 commit 影片檔；是否 commit `promo/` 專案由使用者決定。

## 驗收標準

- `ffprobe` 顯示 1080×1920、30fps、長度 15–25 秒、有音軌。
- 每段字幕各抽一格截圖，逐張檢查：
  - 文字與上表逐字相同，🔥 有正常顯示（不是方框）。
  - 字幕沒有壓到小貓，沒有超出直式安全區。
  - 畫面裡看不到滑鼠游標。
  - K 線上的數字與文字清晰可讀。
- 片尾卡四項資訊齊全：標語、GitHub 連結、credits、「非投資建議」。
- 畫面裡沒有使用者的個人資訊（TradingView 帳號名、瀏覽器書籤列、工作列通知）。有的話裁掉或重錄。

## 風險與備案

- **`ddagrab` 錄不到透明置頂視窗**（未實測）。備案：只錄 TradingView 背景，小貓用 `src/assets/cats.webm`（VP9 + alpha，502×540）在 hyperframes 疊上去並做位移／縮放動畫；右鍵選單改用一張真 app 的截圖。
- **音樂與舞步不同步**：音軌是後製鋪上的，跟實錄的小貓循環起點不一定對齊。可在時間軸上平移音軌起點對拍；對不上也可接受。
- **素材授權**：貓動畫與音樂屬第三方原作者（見 README credits），授權未確認。影片必須保留 credits；收到下架要求時由使用者處理。

## Threads 貼文草稿

主貼文不 tag 任何人。是否到別人的貼文底下回覆，由使用者自己決定與動手。

> 台股開盤，國巨不往上動。
> 沒關係，派出小貓！🐱🐱🐱
>
> 把三隻小貓放到你的看盤畫面上，陪你等漲停。
> Windows 免費下載 👇
> github.com/baabaa-works/three-cat-everything
>
> （非投資建議，小貓只負責跳）

## 規劃階段的假設

- 副顯示器實體解析度約 1920×1200；若實際更小，4:5 裁切後放大超過 1.5 倍會糊，改用更寬的裁切並縮小上下字幕區。
- 「哲哲 = @s178178」「8膩膩 = 巴逆逆 @banini31」是調查時的高可信推斷，未經確認；本片不使用，所以不影響製作。
- 使用者會自己發文，執行 session 不登入、不操作 Threads。
