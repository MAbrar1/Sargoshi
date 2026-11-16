import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/platform/generated/platform_api.g.dart';
import 'package:sargoshi/platform/performance_platform.dart';

void main() {
  test('maps every field, keeping nulls as unknown', () {
    final s = snapshotFromMetrics(
      DeviceMetrics(
        sdkInt: 30,
        isLowRamDevice: false,
        memoryClassMb: 192,
        totalMemMb: 3000,
      ),
    );
    expect(s.sdkInt, 30);
    expect(s.isLowRamDevice, isFalse);
    expect(s.memoryClassMb, 192);
    expect(s.totalMemMb, 3000);
    expect(s.mediaPerformanceClass, isNull);
    expect(s.maxRefreshRateHz, isNull);
    expect(s.thermalStatus, isNull);
  });

  test('maps present optional values', () {
    final s = snapshotFromMetrics(
      DeviceMetrics(
        sdkInt: 35,
        isLowRamDevice: false,
        memoryClassMb: 512,
        totalMemMb: 7800,
        mediaPerformanceClass: 34,
        maxRefreshRateHz: 120,
        thermalStatus: 1,
      ),
    );
    expect(s.mediaPerformanceClass, 34);
    expect(s.maxRefreshRateHz, 120);
    expect(s.thermalStatus, 1);
  });
}
