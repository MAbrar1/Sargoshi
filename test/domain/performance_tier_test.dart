import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/domain/performance/performance_tier.dart';

DeviceSnapshot snap({
  bool lowRam = false,
  int totalMb = 4000,
  int? perfClass,
  double? hz,
  int? thermal,
}) => DeviceSnapshot(
  sdkInt: 34,
  isLowRamDevice: lowRam,
  memoryClassMb: 256,
  totalMemMb: totalMb,
  mediaPerformanceClass: perfClass,
  maxRefreshRateHz: hz,
  thermalStatus: thermal,
);

void main() {
  group('classify', () {
    test('low RAM flag is tier C', () {
      final d = classify(snap(lowRam: true, totalMb: 8000));
      expect(d.tier, PerformanceTier.c);
      expect(d.reason, TierReason.lowRamFlag);
    });

    test('small total memory is tier C', () {
      final d = classify(snap(totalMb: 1800));
      expect(d.tier, PerformanceTier.c);
      expect(d.reason, TierReason.lowTotalMemory);
    });

    test('mid-range phone is tier B', () {
      final d = classify(snap(totalMb: 3800, perfClass: 31, hz: 60));
      expect(d.tier, PerformanceTier.b);
      expect(d.reason, TierReason.defaultClass);
    });

    test('flagship class is tier A', () {
      final d = classify(snap(totalMb: 7500, perfClass: 34, hz: 120));
      expect(d.tier, PerformanceTier.a);
      expect(d.reason, TierReason.flagshipClass);
    });

    test('flagship with severe thermal status drops to B', () {
      final d = classify(
        snap(totalMb: 7500, perfClass: 34, hz: 120, thermal: 3),
      );
      expect(d.tier, PerformanceTier.b);
      expect(d.reason, TierReason.thermalDowngrade);
    });

    test('unknown performance class or refresh rate never earns tier A', () {
      expect(classify(snap(totalMb: 9000, hz: 120)).tier, PerformanceTier.b);
      expect(
        classify(snap(totalMb: 9000, perfClass: 34)).tier,
        PerformanceTier.b,
      );
    });

    test('low refresh rate never earns tier A', () {
      final d = classify(snap(totalMb: 9000, perfClass: 34, hz: 60));
      expect(d.tier, PerformanceTier.b);
    });
  });

  group('effectiveTier', () {
    test('auto keeps the measured tier', () {
      for (final t in PerformanceTier.values) {
        expect(effectiveTier(t, PerformanceMode.auto), t);
      }
    });

    test('always light forces tier C', () {
      for (final t in PerformanceTier.values) {
        expect(effectiveTier(t, PerformanceMode.alwaysLight), PerformanceTier.c);
      }
    });
  });

  group('capabilities (spec section 2)', () {
    test('blur only on tier A', () {
      expect(PerformanceTier.a.blurEnabled, isTrue);
      expect(PerformanceTier.b.blurEnabled, isFalse);
      expect(PerformanceTier.c.blurEnabled, isFalse);
    });

    test('tier C: no shadows, no AI helpers, 30 MB cache, 720p at 10 fps', () {
      const c = PerformanceTier.c;
      expect(c.shadowsEnabled, isFalse);
      expect(c.aiHelpersEnabled, isFalse);
      expect(c.imageCacheMaxMb, 30);
      expect(c.maxShareHeight, 720);
      expect(c.maxShareFps, 10);
    });

    test('share resolution: 1080p only on tier A', () {
      expect(PerformanceTier.a.maxShareHeight, 1080);
      expect(PerformanceTier.b.maxShareHeight, 720);
    });
  });
}
