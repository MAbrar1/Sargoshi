import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/measured/measured.dart';
import '../../domain/performance/performance_tier.dart';
import '../../platform/performance_platform.dart';

const String kTierSource =
    'ActivityManager + MediaPerformanceClass + DisplayManager + PowerManager';

final deviceSnapshotSourceProvider = Provider<DeviceSnapshotSource>(
  (ref) => PigeonDeviceSnapshotSource(),
);

/// Measured once per process. If the platform call fails the result is
/// unknown, never a guessed tier.
final tierMeasurementProvider = FutureProvider<Measured<TierDecision>>((
  ref,
) async {
  final source = ref.watch(deviceSnapshotSourceProvider);
  final at = DateTime.now().toUtc();
  try {
    final snapshot = await source.read();
    return Measured.of(classify(snapshot), source: kTierSource, measuredAt: at);
  } on Object {
    return Measured.unknown(source: kTierSource, measuredAt: at);
  }
});

/// Rendering needs some tier even when the measurement is unknown. Tier B is
/// the spec default (no blur, flat surfaces).
PerformanceTier tierForRendering(Measured<TierDecision>? m) =>
    m?.value?.tier ?? PerformanceTier.b;
