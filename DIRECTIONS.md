# EsperaVox // Camera Cinematics & Spline Motion Guide
> **A Comprehensive Guide to Keyframe Animation, Spline Motion, and Automated Video Generation**
> **Written by:** Esperanza for Jack

---

## 1. Overview

In 3D voxel art, motion transforms static models into living, breathing scenes. **EsperaVox** and **LinuxVox Core** use **Keyframe Cinematics with Catmull-Rom Spline Interpolation** to eliminate hand-jitter, creating silky-smooth, broadcast-quality crane, dolly, and fly-through videos from your voxel creations.

---

## 2. How the Spline System Works

Rather than moving in rigid, straight lines between camera angles, the camera glides along curved paths mathematically modeled after physical camera dolly tracks.

```
Keyframe 1                Keyframe 2 (Rising)
  (Low Entry)                 * * * *
      *                     *         *
       *                  *             *
        *               *                *
          * * * * * * *                    *
                                            *
                                   Keyframe 3 (High Crane)
```

### The Three Core Components:
1. **Catmull-Rom Spline Interpolation:**
   * Uses 4 consecutive control points $(p_0, p_1, p_2, p_3)$ to calculate a smooth, tension-balanced cubic polynomial curve.
   * Guarantees the camera passes directly through every keyframe without sharp corners.
2. **Smoothstep Easing ($3t^2 - 2t^3$):**
   * Eliminates robotic, linear velocity. The camera gently accelerates into turns and cushions smoothly to a stop, mimicking a heavy camera rig with physical inertia.
3. **Angular Wrap-Around Normalization:**
   * Prevents the camera from doing an accidental 360° reverse spin when passing across the $0^\circ \leftrightarrow 360^\circ$ boundary.
4. **Seamless Looping:**
   * When looping is enabled, the spline wraps around from the final keyframe back to the first, creating hypnotic, infinite video loops.

---

## 3. Command-Line Video Generator (`vox2video`)

You can generate videos from any `.vox` model headlessly with the included `vox2video` tool.

### Syntax
```bash
/home/jack/Desktop/EsperaVox/target/debug/vox2video <input.vox> <output.mp4> [options]
```

| Flag | Description | Default |
|---|---|---|
| `--fps <N>` | Video framerate (e.g., `30` or `60`) | `30` |
| `--seconds <F>` | Total duration in seconds | `4.0` |
| `--width <N>` | Video horizontal resolution in pixels | `1280` |
| `--height <N>` | Video vertical resolution in pixels | `720` |
| `--mode <crane\|orbit>` | Camera choreography style | `crane` |
| `--bg <mode>` | `transparent`, `studio`, `white`, `black`, or `#RRGGBB` | `studio` |
| `--bg-image <path>` | 2D image plane backdrop file (PNG/JPG) | None |

---

## 4. Practical Examples

### Example 1: The Dramatic Crane & Dolly Swoop (Recommended)
Performs a multi-elevation Hollywood-style sweep—starting low at the entrance, swooping upward around the flanks, and overlooking the towers from an aerial crane angle before looping back:
```bash
cd /home/jack/Desktop/EsperaVox
cargo run --bin vox2video -- examples/Castle_Keep.vox Castle_Crane.mp4 --seconds 4.0 --fps 30 --width 960 --height 720
```

### Example 2: The 360° Hero Orbit Loop
A perfectly steady, continuous circular orbit around the model:
```bash
cargo run --bin vox2video -- examples/Retro_Bot.vox Bot_Orbit.mp4 --mode orbit --seconds 5.0 --fps 30
```

### Example 3: High-Definition 60 FPS Showcase
For portfolio or social media presentation with ultra-smooth motion:
```bash
cargo run --bin vox2video -- examples/Floating_Island.vox Island_60fps.mp4 --seconds 6.0 --fps 60 --width 1920 --height 1080
```

### Example 4: Batch Video Generation for an Entire Folder
Automatically turn every `.vox` file in a directory into an MP4 video:
```bash
for file in examples/*.vox; do
    name=$(basename "$file" .vox)
    echo "Rendering video for $name..."
    /home/jack/Desktop/EsperaVox/target/debug/vox2video "$file" "${name}_cinematic.mp4" --seconds 4.0 --fps 30
done
```

---

## 5. The Interactive Studio Director Workflow

When using the graphical **EsperaVox** desktop studio, setting up camera moves follows this intuitive rhythm:

1. **Find Your Angle:** Use the standard orbit controls (Right-Click Drag, Pan, Wheel Zoom) to position the camera at your opening shot.
2. **Drop a Keyframe:** Click **Capture Keyframe** on the bottom timeline.
3. **Move to the Next Beat:** Navigate to your next viewpoint (e.g. rising above the roof or closing in on a detail) and drop Keyframe 2.
4. **Close the Loop:** To create an infinite loop, orbit back to your starting area and toggle **Looping**.
5. **Live Preview:** Tap **Play (▶)** to see the camera perform the smooth swoop directly in the viewport.
6. **Export Video:** Tap **Render MP4** to generate your finished video file.

---

## 6. Generated Assets in this Workspace

* 🎥 **[Castle_Cinematic.mp4](file:///home/jack/Desktop/EsperaVox/Castle_Cinematic.mp4)**: 4.00-second 30 FPS H.264 video demonstrating the multi-stage crane swoop around `Castle_Keep.vox`.
* 🛠️ **[vox2video binary](file:///home/jack/Desktop/EsperaVox/target/debug/vox2video)**: Standalone video rendering CLI.
* 📦 **[crates/linuxvox-core/src/cinematic.rs](file:///home/jack/Desktop/EsperaVox/crates/linuxvox-core/src/cinematic.rs)**: Core spline & easing engine.
