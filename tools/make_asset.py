"""Turn the green-screen source clip into a transparent VP9 WebM.

Usage:  .venv/Scripts/python.exe -I tools/make_asset.py [raw/source.mp4]

Outputs:
  src/assets/cats.webm         VP9 + alpha, Opus audio, cropped to the cats
  raw/preview/*.png            composites on dark / light / red backgrounds
  app-icon.png                 1024x1024 icon from one keyed frame
"""
import subprocess
import sys
from pathlib import Path

import imageio_ffmpeg
import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SRC = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "raw" / "source.mp4"
OUT = ROOT / "src" / "assets" / "cats.webm"
PREVIEW = ROOT / "raw" / "preview"
FF = imageio_ffmpeg.get_ffmpeg_exe()

W, H, FPS = 720, 1280, 30
# Key thresholds on "green dominance" d = g - max(r, b).
# Background sits around 240; cat fur is <= ~20.
KEY_LO, KEY_HI = 45, 120
PAD = 10


def read_frames():
    cmd = [FF, "-v", "error", "-i", str(SRC), "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE)
    size = W * H * 3
    frames = []
    while True:
        buf = p.stdout.read(size)
        if len(buf) < size:
            break
        frames.append(np.frombuffer(buf, np.uint8).reshape(H, W, 3))
    p.wait()
    return frames


def key(rgb):
    f = rgb.astype(np.int16)
    r, g, b = f[..., 0], f[..., 1], f[..., 2]
    d = g - np.maximum(r, b)
    alpha = 1.0 - np.clip((d - KEY_LO) / (KEY_HI - KEY_LO), 0.0, 1.0)
    # Despill: never let green exceed the brighter of red/blue.
    g2 = np.minimum(g, np.maximum(r, b))
    out = np.stack([r, g2, b], axis=-1).clip(0, 255).astype(np.uint8)
    a = (alpha * 255).round().astype(np.uint8)
    # Kill isolated speckles in the background.
    a[a < 12] = 0
    return np.dstack([out, a])


def main():
    frames = read_frames()
    print(f"read {len(frames)} frames")
    keyed = [key(f) for f in frames]

    union = np.zeros((H, W), bool)
    for k in keyed:
        union |= k[..., 3] > 25
    ys, xs = np.where(union)
    y0, y1 = max(ys.min() - PAD, 0), min(ys.max() + PAD, H)
    x0, x1 = max(xs.min() - PAD, 0), min(xs.max() + PAD, W)
    # even dimensions for yuv420
    if (y1 - y0) % 2:
        y1 -= 1
    if (x1 - x0) % 2:
        x1 -= 1
    cw, ch = x1 - x0, y1 - y0
    print(f"crop x={x0}..{x1} y={y0}..{y1} -> {cw}x{ch}")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        FF, "-v", "error", "-y",
        "-f", "rawvideo", "-pix_fmt", "rgba", "-s", f"{cw}x{ch}", "-r", str(FPS), "-i", "-",
        "-i", str(SRC),
        "-map", "0:v", "-map", "1:a",
        # 540px tall is plenty for a 120-900px window and keeps decode cheap.
        "-vf", "scale=-2:540:flags=lanczos",
        "-c:v", "libvpx-vp9", "-pix_fmt", "yuva420p", "-b:v", "0", "-crf", "30",
        "-auto-alt-ref", "0", "-row-mt", "1", "-deadline", "good", "-cpu-used", "2",
        "-c:a", "libopus", "-b:a", "96k",
        "-shortest", str(OUT),
    ]
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE)
    for k in keyed:
        p.stdin.write(np.ascontiguousarray(k[y0:y1, x0:x1]).tobytes())
    p.stdin.close()
    if p.wait() != 0:
        raise SystemExit("ffmpeg encode failed")
    print(f"wrote {OUT} ({OUT.stat().st_size / 1024:.0f} KiB)")

    PREVIEW.mkdir(parents=True, exist_ok=True)
    bgs = {"dark": (18, 18, 22), "light": (245, 245, 245), "red": (200, 30, 40)}
    picks = [len(keyed) // 6, len(keyed) // 2, len(keyed) * 5 // 6]
    for name, color in bgs.items():
        tiles = []
        for i in picks:
            im = Image.fromarray(keyed[i][y0:y1, x0:x1], "RGBA")
            bg = Image.new("RGBA", im.size, color + (255,))
            tiles.append(Image.alpha_composite(bg, im))
        sheet = Image.new("RGBA", (cw * len(tiles), ch))
        for j, t in enumerate(tiles):
            sheet.paste(t, (j * cw, 0))
        sheet.convert("RGB").save(PREVIEW / f"{name}.png")

    # icon: square crop around the cats from a frame where they stand together
    im = Image.fromarray(keyed[int(FPS * 2.0)][y0:y1, x0:x1], "RGBA")
    bbox = im.getchannel("A").point(lambda v: 255 if v > 25 else 0).getbbox()
    im = im.crop(bbox)
    side = max(im.size)
    sq = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    sq.paste(im, ((side - im.width) // 2, (side - im.height) // 2))
    sq.resize((1024, 1024), Image.LANCZOS).save(ROOT / "app-icon.png")
    print("previews + icon written")


if __name__ == "__main__":
    main()
