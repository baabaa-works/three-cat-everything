import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import catsUrl from "./assets/cats.webm";

type Group = {
  id: number;
  height: number;
  opacity: number;
  locked: boolean;
  passthrough: boolean;
};

const win = getCurrentWindow();
const id = Number(win.label.split("-")[1]);
const video = document.getElementById("cats") as HTMLVideoElement;
video.src = catsUrl;
void video.play().catch(() => {});

let locked = false;

function apply(g: Group) {
  locked = g.locked;
  video.style.opacity = String(g.opacity);
  document.body.style.cursor = g.locked ? "pointer" : "grab";
}

invoke<Group | null>("get_group").then((g) => g && apply(g));
void listen<Group>("group-changed", (e) => {
  if (e.payload.id === id) apply(e.payload);
});

// ---- music: the soundtrack lives in the video, so it never drifts ----
async function setMusic(on: boolean) {
  video.muted = !on;
  if (on) void video.play().catch(() => {});
  await invoke("set_music", { playing: on });
}
void listen<number>("toggle-music", (e) => {
  if (e.payload === id) void setMusic(video.muted);
});
// Only one group plays at a time.
void listen<number>("music-owner", (e) => {
  if (e.payload !== id && !video.muted) {
    video.muted = true;
    void invoke("set_music", { playing: false });
  }
});

// ---- click vs drag ----
const DRAG_THRESHOLD = 4;
let down: { x: number; y: number } | null = null;

window.addEventListener("pointerdown", (e) => {
  if (e.button !== 0) return;
  down = { x: e.screenX, y: e.screenY };
});
window.addEventListener("pointermove", (e) => {
  if (!down || locked) return;
  if (Math.hypot(e.screenX - down.x, e.screenY - down.y) > DRAG_THRESHOLD) {
    down = null;
    void win.startDragging();
  }
});
window.addEventListener("pointerup", (e) => {
  if (e.button !== 0) return;
  if (down) void setMusic(video.muted);
  down = null;
});

// ---- wheel zoom ----
let scaling = false;
window.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault();
    if (locked || scaling || e.deltaY === 0) return;
    scaling = true;
    invoke("scale_group", { factor: e.deltaY < 0 ? 1.1 : 1 / 1.1 }).finally(() => {
      scaling = false;
    });
  },
  { passive: false },
);

// ---- native context menu ----
window.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  void invoke("show_context_menu");
});
