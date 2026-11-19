# Heist online and arena readability

The spawn pass keeps the original 25 positions per team and adds slots on clear lanes by each base. The two-flag reference has 33 per base. The three-flag reference has 57 per base. The four-flag reference has 57 at two bases and 33 at the others, where adjacent walls limit the straight approach. Legacy browser fixtures confirmed 20 separated starts on every layout. The lighter olive turf retains the original texture with softer detail. Contrasting wall faces and painted doorway marks make the bank approach clearer. [Base approach](approach.png) · [Vault](vault.png) · [Hatch](hatch.png)

Two native clients joined the four-flag Heist on the public relay with 18 bots and entered play. The host and guest showed the same running clock and Dazza position. A browser guest also joined a native four-flag host and reached the running round with six bots. [Browser guest view](browser-guest.png)

Network tests confirmed the guest receives the host's banks and hatch state and that the host consumes a guest's drink once. Rust players join before the round starts and need protocol 31.

The workspace passed 747 tests with 3 ignored. Automated runs verified six crate vaults, nine tunnel exits with the hatch closed behind the carrier, and 20-participant bot banks on all three layouts. The legacy browser ran the escort, timeout, tailgating and repeat-use checks again. A three-flag bot repeatedly turned back toward the crate just after crossing the vault. Restricting the shortcut trigger to the approach side cleared that stall on the replay. The shortcut and teammate-pass rules are available for balance feedback during a full human match.
