import 'package:sargoshi/application/vault/attempt_limiter.dart';

/// Clock the test controls.
final class FakeAttemptClock implements AttemptClock {
  int elapsedMs = 1000;
  int boot = 1;

  void advance(Duration d) => elapsedMs += d.inMilliseconds;

  @override
  Future<AttemptTime> now() async => (elapsedMs: elapsedMs, boot: boot);
}
