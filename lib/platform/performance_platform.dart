import '../domain/performance/performance_tier.dart';
import 'generated/platform_api.g.dart';

/// Maps the generated Pigeon type to the pure domain type.
DeviceSnapshot snapshotFromMetrics(DeviceMetrics m) => DeviceSnapshot(
  sdkInt: m.sdkInt,
  isLowRamDevice: m.isLowRamDevice,
  memoryClassMb: m.memoryClassMb,
  totalMemMb: m.totalMemMb,
  mediaPerformanceClass: m.mediaPerformanceClass,
  maxRefreshRateHz: m.maxRefreshRateHz,
  thermalStatus: m.thermalStatus,
);

final class PigeonDeviceSnapshotSource implements DeviceSnapshotSource {
  PigeonDeviceSnapshotSource([PerformanceApi? api])
    : _api = api ?? PerformanceApi();

  final PerformanceApi _api;

  @override
  Future<DeviceSnapshot> read() async =>
      snapshotFromMetrics(await _api.getDeviceMetrics());
}
