/// Performance tiers (spec section 2). Pure Dart: no platform imports.
///
/// The raw facts are measured in Kotlin at first launch. This file only turns
/// them into a tier. Thresholds are PROVISIONAL: see docs/adr/ADR-007.
library;

enum PerformanceTier { a, b, c }

enum PerformanceMode { auto, alwaysLight }

/// Why a tier was chosen, shown in the diagnostics row.
enum TierReason {
  lowRamFlag,
  lowTotalMemory,
  flagshipClass,
  thermalDowngrade,
  defaultClass,
}

/// Raw device facts. Null means the platform could not provide the value.
final class DeviceSnapshot {
  const DeviceSnapshot({
    required this.sdkInt,
    required this.isLowRamDevice,
    required this.memoryClassMb,
    required this.totalMemMb,
    this.mediaPerformanceClass,
    this.maxRefreshRateHz,
    this.thermalStatus,
  });

  final int sdkInt;
  final bool isLowRamDevice;
  final int memoryClassMb;
  final int totalMemMb;

  /// Build.VERSION.MEDIA_PERFORMANCE_CLASS. 0 means not declared, null means
  /// the Android version is too old to report it.
  final int? mediaPerformanceClass;
  final double? maxRefreshRateHz;
  final int? thermalStatus;
}

final class TierDecision {
  const TierDecision(this.tier, this.reason);
  final PerformanceTier tier;
  final TierReason reason;
}

/// Totals reported by ActivityManager are below the marketing figure
/// (a "2 GB" phone reports roughly 1.8 GB), so these are not round numbers.
const int kTierCBelowTotalMemMb = 2560;
const int kTierAMinTotalMemMb = 5500;
const int kTierAMinPerformanceClass = 33; // Android 13 performance class
const double kTierAMinRefreshHz = 90;
const int kThermalSevere = 3; // PowerManager.THERMAL_STATUS_SEVERE

TierDecision classify(DeviceSnapshot s) {
  if (s.isLowRamDevice) {
    return const TierDecision(PerformanceTier.c, TierReason.lowRamFlag);
  }
  if (s.totalMemMb < kTierCBelowTotalMemMb) {
    return const TierDecision(PerformanceTier.c, TierReason.lowTotalMemory);
  }
  final perfClass = s.mediaPerformanceClass;
  final refresh = s.maxRefreshRateHz;
  final flagship =
      perfClass != null &&
      perfClass >= kTierAMinPerformanceClass &&
      s.totalMemMb >= kTierAMinTotalMemMb &&
      refresh != null &&
      refresh >= kTierAMinRefreshHz;
  if (flagship) {
    final thermal = s.thermalStatus;
    if (thermal != null && thermal >= kThermalSevere) {
      return const TierDecision(PerformanceTier.b, TierReason.thermalDowngrade);
    }
    return const TierDecision(PerformanceTier.a, TierReason.flagshipClass);
  }
  return const TierDecision(PerformanceTier.b, TierReason.defaultClass);
}

/// Settings > Performance: "Always light" forces the lightest rendering rules.
PerformanceTier effectiveTier(PerformanceTier measured, PerformanceMode mode) =>
    mode == PerformanceMode.alwaysLight ? PerformanceTier.c : measured;

/// What each tier turns on or off (spec section 2 and 5.4).
extension TierCapabilities on PerformanceTier {
  bool get blurEnabled => this == PerformanceTier.a;
  bool get shadowsEnabled => this != PerformanceTier.c;
  bool get aiHelpersEnabled => this != PerformanceTier.c;

  /// Null means the Flutter default. Only tier C has a spec value.
  int? get imageCacheMaxMb => this == PerformanceTier.c ? 30 : null;

  int get maxShareHeight => this == PerformanceTier.a ? 1080 : 720;
  int get maxShareFps => this == PerformanceTier.c ? 10 : 15;
}

/// Source of raw facts. The platform layer implements it.
abstract interface class DeviceSnapshotSource {
  Future<DeviceSnapshot> read();
}
