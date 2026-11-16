import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/performance/tier_provider.dart';
import 'package:sargoshi/domain/performance/performance_tier.dart';

class _Fake implements DeviceSnapshotSource {
  _Fake(this._result);
  final Future<DeviceSnapshot> Function() _result;

  @override
  Future<DeviceSnapshot> read() => _result();
}

void main() {
  test('success is a known measurement with a source', () async {
    final c = ProviderContainer(
      overrides: [
        deviceSnapshotSourceProvider.overrideWithValue(
          _Fake(
            () async => const DeviceSnapshot(
              sdkInt: 34,
              isLowRamDevice: true,
              memoryClassMb: 128,
              totalMemMb: 1800,
            ),
          ),
        ),
      ],
    );
    addTearDown(c.dispose);
    final m = await c.read(tierMeasurementProvider.future);
    expect(m.isKnown, isTrue);
    expect(m.value!.tier, PerformanceTier.c);
    expect(m.source, kTierSource);
  });

  test('platform failure is unknown, not a guessed tier', () async {
    final c = ProviderContainer(
      overrides: [
        deviceSnapshotSourceProvider.overrideWithValue(
          _Fake(() async => throw StateError('no platform')),
        ),
      ],
    );
    addTearDown(c.dispose);
    final m = await c.read(tierMeasurementProvider.future);
    expect(m.isKnown, isFalse);
    expect(tierForRendering(m), PerformanceTier.b);
    expect(tierForRendering(null), PerformanceTier.b);
  });
}
