# Latest recovery checkpoint

[More tip flex — reviewed and approved for commit](2026-10-15-tip-flex/CHECKPOINT.md).

The user approved committing and pushing the accumulated graphics, character, item physics, held motion, Q weapon-switch labels, silicone models and tip-flex changes on 15 October 2026. The implementation and required assets are included together.

Compressed motion clips, static screenshots, scripts and recovery notes are versioned. Raw numbered motion-frame sequences remain local to avoid adding nearly 2 GB to Git. Frame-by-frame gallery controls require those local sequences; animated WebP comparisons remain available after cloning. Regenerate sequences using the review scripts when needed.

Validation: latest app tests passed (182 passed, 1 ignored), release build passed, actual held/drop captures checked. Fresh performance comparison: 8.114 ms before, 8.083 ms after. Earlier core validation: 411 passed; core has not changed since that check.
