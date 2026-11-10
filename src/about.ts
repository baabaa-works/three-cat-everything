import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";

getVersion().then((v) => {
  document.getElementById("ver")!.textContent = `v${v}`;
});

document.querySelectorAll<HTMLAnchorElement>("a[data-href]").forEach((a) => {
  a.addEventListener("click", () => void openUrl(a.dataset.href!));
});
