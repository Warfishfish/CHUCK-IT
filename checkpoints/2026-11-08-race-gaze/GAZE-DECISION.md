# Gaze approach

The existing camera already exposes raw pitch and yaw. Pitch is currently absent from player snapshots. Cosmetic camera lean, hit shake and drunk effects are separate and should not become gaze input. Existing yaw continues to turn the whole character.

The Face length system already gathers the head, eyes, pupils, expressions, hair and hats under one parent. Rotate that parent around a small neck pivot after existing body animation. Keep the shape slider's scaling centred on the head. This costs one transform update per character, with no new mesh, bones, render pass or physics.

Use the existing 20 Hz player snapshots. Add one signed pitch byte over the existing camera range ±1.45 radians, neutral exactly zero. Quantisation error is at most 0.33 degrees. At 20 Hz the extra payload is 20 bytes per second per sending player, before the existing relay envelope and fan-out. Protocol 28 deliberately refuses old versions rather than interpreting a mismatched packet.

Draw up to 50.4 degrees upward and 41.3 degrees downward. Exponential smoothing at response 12/s reaches about 95% in 250 ms. Head pose composes under the body's existing charge, throw, hit, locomotion and carry animations. Knockdown/seated poses return to neutral; hit-stop holds the current head. Packets older than 500 ms ease to neutral. New/rebuilt characters start neutral. Preview and lobby heads remain neutral.

Gaze never changes camera controls, shot origin, throw velocity, targeting, hitboxes, movement, authority or scoring. Keep hands/shirt names attached to their existing parents.

Research performed locally with the current Sol model. Astra was not invoked. The inspected hierarchy supports the smallest implementation directly; no Blender asset change or paid service is needed.
