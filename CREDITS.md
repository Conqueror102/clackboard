# Credits

Clackboard is built on the work of others.

## Code

- **[Hushboard](https://github.com/tekcify/hushboard)** by Tekcify and contributors (MIT). Clackboard started as a fork; the keyboard lock, tray panel, island and design system come from it.

## Switch sounds

The packs live in `src-tauri/resources/packs/keyboard/`, one folder per switch. Each folder keeps the license its sounds came with. Clackboard's own code is MIT; the sound files keep their original licenses, so anyone redistributing a pack's files must follow that pack's license.

| Packs | Source | License |
| --- | --- | --- |
| Cherry MX Black, Blue, Brown (ABS, PBT), Cherry MX Red, EG Oreo, EG Oreo Classic, EG Crystal Purple, Topre Purple Hybrid | [Mechvibes](https://github.com/hainguyents13/mechvibes) by Hải Nguyễn | MIT (`packs/LICENSE-mechvibes.txt`) |
| Alpaca, Gateron Black Ink, Gateron Red Ink, Holy Panda, Cherry MX Black / Blue / Brown (kbsim) | [kbsim](https://github.com/tplai/kbsim) by Thomas Lai, via [Keyboard Sounds](https://github.com/nathan-fiscaletti/keyboardsounds) | MIT (`LICENSE.txt` in each pack) |
| NovelKeys Cream, Telios V2, Cherry MX Speed Silver, Typewriter (Mechvibes community); Banana Split, SteelSeries Apex Pro TKL (Akira); Logitech G915 TKL (Keyboard Sounds) | [Keyboard Sounds](https://github.com/nathan-fiscaletti/keyboardsounds) by Nathan Fiscaletti and contributors | GPL-3.0 (`LICENSE.txt` in each pack) |
| IBM Model M | [bucklespring](https://github.com/zevv/bucklespring) by Ico Doornekamp | GPL-2.0 (`LICENSE.txt` in the pack) |

The Keyboard Sounds and bucklespring recordings were converted with `tools/pack-import` (mixed to mono, silence trimmed, joined into one file per pack); the website's demo copies were made with `tools/site-packs`. Clackboard reads the Mechvibes pack format (V1 and V2), so more packs can be added from the panel.

## Icons and fonts

- **Icons**: Nucleo Glass (free set, Nucleo license), see `design-system/project/assets/IconsGlass/README.md`.
- Fonts and other third-party notices: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
