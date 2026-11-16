# Bundled fonts

| Family | Use | Files | Licence |
|---|---|---|---|
| Inter | All UI text | Inter-Variable.ttf (weights 400 to 700, optical size pinned at 14) | SIL OFL 1.1 |
| Instrument Serif | Wordmark, onboarding, empty states, large Trust status | Regular, Italic | SIL OFL 1.1 |
| JetBrains Mono | Safety numbers and fingerprints only | Regular | SIL OFL 1.1 |

Total payload: 193,008 bytes (budget 250 KB). Latin subset: Basic Latin, Latin-1, spaces, dashes, quotes and common punctuation.

## Provenance
Source: https://github.com/google/fonts at commit cc39c4c2c304bc368c9b7c311ea2bcc6013299fe (fetched 2026-11-16).

| Source file | sha256 |
|---|---|
| ofl/inter/Inter[opsz,wght].ttf | 29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031 |
| ofl/instrumentserif/InstrumentSerif-Regular.ttf | 498efd461f6ddfcb7a111bf9a565709d2085d48201d501ead960d93e84ffbb88 |
| ofl/instrumentserif/InstrumentSerif-Italic.ttf | 08939b8bdf534afec24ae0ef5e03f948940cd9a8fe08e7fecbad040e62327385 |
| ofl/jetbrainsmono/JetBrainsMono[wght].ttf | 48715a42ec242c21e9f02692891e147d022299a52e48d5e413e1a942193ffeda |

## Rebuild
    pip install fonttools
    git clone --depth 1 --filter=blob:none --sparse https://github.com/google/fonts /tmp/gf
    (cd /tmp/gf && git sparse-checkout set ofl/inter ofl/instrumentserif ofl/jetbrainsmono)
    python3 tool/fonts/build_fonts.py /tmp/gf assets/fonts

## Known gaps
- Instrument Serif lacks some Latin-1 symbols (for example ± ² ³ µ ¼ ½ ¾). It is used for short headlines only.
- JetBrains Mono has no thin space (U+2009). Safety-number groups must be separated by layout spacing, not by a space character (Stage 3).
