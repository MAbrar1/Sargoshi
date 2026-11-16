// Pigeon definition for the typed platform channel (spec section 2).
// Regenerate with: dart run pigeon --input pigeons/platform_api.dart
// then: dart format lib/platform/generated
import 'package:pigeon/pigeon.dart';

@ConfigurePigeon(
  PigeonOptions(
    dartOut: 'lib/platform/generated/platform_api.g.dart',
    kotlinOut: 'android/app/src/main/kotlin/app/sargoshi/PlatformApi.g.kt',
    kotlinOptions: KotlinOptions(package: 'app.sargoshi'),
    dartPackageName: 'sargoshi',
  ),
)
/// Raw device facts measured in Kotlin at first launch. Null means the
/// platform could not provide the value (older Android version or API error).
class DeviceMetrics {
  DeviceMetrics({
    required this.sdkInt,
    required this.isLowRamDevice,
    required this.memoryClassMb,
    required this.totalMemMb,
    this.mediaPerformanceClass,
    this.maxRefreshRateHz,
    this.thermalStatus,
  });

  int sdkInt;
  bool isLowRamDevice;
  int memoryClassMb;
  int totalMemMb;

  /// Build.VERSION.MEDIA_PERFORMANCE_CLASS (API 31+). 0 = not declared.
  int? mediaPerformanceClass;

  /// Highest refresh rate among the display's supported modes.
  double? maxRefreshRateHz;

  /// PowerManager thermal status 0..6 (API 29+).
  int? thermalStatus;
}

@HostApi()
abstract class PerformanceApi {
  DeviceMetrics getDeviceMetrics();
}
