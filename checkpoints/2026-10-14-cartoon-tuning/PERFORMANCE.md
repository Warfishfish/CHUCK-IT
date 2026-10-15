# Cartoon refinement performance

Apple M1 Pro, 14 GPU cores, 16 GB. Release build at 1280 × 800. Same scene, camera, fixed 60 Hz simulation and capture frame within each pair; 300 warmup frames, 600 wall-clock frame intervals, AutoNoVsync. One game process at a time, no concurrent compilation. Measurements exclude screenshot readback.

| View | Before mean ms | After mean ms | Change | Before P95 ms | After P95 ms | After FPS |
|---|---:|---:|---:|---:|---:|---:|
| normal | 6.284 | 6.439 | +2.5% | 7.411 | 7.535 | 155.3 |
| gumdrop | 6.429 | 6.563 | +2.1% | 7.568 | 7.631 | 152.4 |
| yard | 7.289 | 7.393 | +1.4% | 8.458 | 8.669 | 135.3 |
| stress | 7.847 | 7.734 | -1.4% | 8.774 | 8.534 | 129.3 |
| current | 6.060 | 6.130 | +1.2% | 7.061 | 7.274 | 163.1 |

11-bot Heist stayed at a similar level: 127.4 → 129.3 FPS; mean 7.847 → 7.734 ms; P95 8.774 → 8.534 ms. Neither stress run had a measured frame above 16.67 ms. Small differences should not be interpreted as a guaranteed speed increase.

Stress mesh assets remain 2,897, mesh entities 7,736, CPU image data 42,129,816 bytes. Materials 911 → 912 (palette-dependent caching). No geometry or texture assets added. These counts are not draw calls or total GPU memory.

Current-style regression is visually unchanged; 18 of 1,024,000 pixels differ. The new styling is restricted to Pop.

These short runs do not establish all hardware, thermal or gameplay performance. No engagement improvement is claimed. Raw metrics and capture logs remain in evidence/.
