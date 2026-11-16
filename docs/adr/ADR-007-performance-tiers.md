# ADR-007: Performance tier thresholds

Status: PROVISIONAL. The thresholds below are my starting values, not measurements. They must be calibrated on the device matrix in spec section 10 before Stage 10.

## Decision
- Facts are measured in Kotlin at first launch (`PerformanceApiImpl`) and returned through Pigeon as raw values. Unknown values stay null.
- Classification is a pure Dart function (`classify`) so it is unit-tested in CI.
- Tier C: `isLowRamDevice`, or total memory below 2560 MB.
- Tier A: media performance class at least 33, total memory at least 5500 MB, max refresh rate at least 90 Hz, and thermal status below SEVERE (3). Anything unknown never earns tier A.
- Tier B: everything else, and the rendering fallback when the measurement is unknown.
- Settings > Performance "Always light" forces tier C rendering rules.

## Why total memory, not only memory class
The spec lists memory class as an input. Memory class is the per-app heap limit and varies by OEM. Total memory is a better signal for "2 GB phone", so both are collected and total memory is used. ActivityManager reports less than the marketing figure (a 2 GB phone is roughly 1.8 GB), hence 2560 and 5500 rather than round numbers.

## Not done
- No on-device calibration. TODO(verify) on a 2 GB Android Go phone, a mid-range phone and a flagship.
- The "Always light" setting is not stored yet. It needs the settings store from Stage 2 or 8.
- The diagnostics row that shows the inputs is not built yet (Settings, Stage 8).
