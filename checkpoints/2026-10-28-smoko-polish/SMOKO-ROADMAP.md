# Smoko-first development direction

Agreed with the user on 28–29 October 2026: Smoko / Bush Servo is the primary map for performance and gameplay polish. Improve and playtest it first, then deliberately transfer proven improvements to other maps. Preserve the distinct maps and their objectives.

1. First pass: remove hidden scenery update work; contextual pickup, carry and delivery instructions; preserve rules. Establish matched capture/timing evidence and recovery checkpoint.
2. Browser multiplayer: measure host and guest frame pacing on actual devices; verify mouse capture, recovery and movement. Separate network corrections from rendering stalls before further optimisation.
3. Carrying and routes: playtest pickup reach, interruptions, approaches to the shelter, bot deliveries and recovery. Tune only from observed friction; preserve one-score-per-delivery and host authority.
4. Feedback and readability: make targets, delivery progress and successful rescues easier to understand, without obscuring fights or relying only on colour.
5. Scene performance and finish: profile environment geometry/materials and loading; improve the Bush Servo concept match within an explicit browser budget.
6. Transfer: after review, reuse shared UI, animation, performance and feedback improvements in other maps. Keep map-specific terrain, objectives and balance distinct.

Each pass ends with evidence, limitations, local test command and review before commit, push or publishing.
