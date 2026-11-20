# Teddy Heist hit recovery and grass footsteps

Prototype 0.23.46 clears the interrupted bot's target, dodge, glance and cached route after a stun. A Heist bot returns to carrying, raiding, guarding or recovering its own teddy. A bot with full hands or no available enemy teddy heads to its defensive post. Bots can still go after a visible opponent carrying their own team's teddy. A bot winds up from a planted stance, and Rust walking orientation no longer uses velocity accumulated during the hit pose.

The legacy browser regression sends a real thrown teddy through the collision and stun rules. Before the fix the old dodge and glance were still active on recovery, and a bot with full hands had no objective. With the fix both timers cleared and it had a guard objective. A Rust regression also seeds an active combat target and verifies that stun recovery drops it.

Three local first-person sounds trigger by distance travelled on Heist's grass at ground level. They are gated by the sound setting and the active round. Stopping, being airborne or in the tunnel, being stunned and teleport corrections do not produce grass steps. The recording is by Fission9 from Freesound, CC0. Attribution is in the shipped assets.

Rust workspace tests passed with 753 passing and three ignored. Twenty-participant Heist tests recorded 8, 12 and 16 banks on the 2-, 3- and 4-flag layouts. The legacy browser decoded and played all three WAVs during a controlled walk and stayed quiet when stationary, underground or muted.
