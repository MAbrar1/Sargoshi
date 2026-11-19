import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/vault/attempt_limiter.dart';
import 'package:sargoshi/domain/vault/attempt_policy.dart';
import 'package:sargoshi/domain/vault/vault_lock.dart';

import '../helpers/fake_attempts.dart';

final class _Verifier implements PinVerifier {
  _Verifier(this.answer);
  PinCheck answer;
  Object? failure;
  int calls = 0;

  @override
  Future<PinCheck> check(String pin) async {
    calls++;
    if (failure != null) throw failure!;
    return answer;
  }
}

void main() {
  late InMemoryAttemptStore store;
  late FakeAttemptClock clock;
  late AttemptLimiter limiter;
  late VaultAttemptGuard guard;

  setUp(() {
    store = InMemoryAttemptStore();
    clock = FakeAttemptClock();
    limiter = AttemptLimiter(store: store, clock: clock);
    guard = VaultAttemptGuard(limiter);
  });

  Future<void> failTimes(int n, _Verifier v) async {
    for (var i = 0; i < n; i++) {
      await guard.check(v, '9999');
    }
  }

  test('delay schedule', () {
    expect(delayAfter(0), Duration.zero);
    expect(delayAfter(4), Duration.zero);
    expect(delayAfter(5), const Duration(seconds: 30));
    expect(delayAfter(6), const Duration(minutes: 1));
    expect(delayAfter(7), const Duration(minutes: 5));
    expect(delayAfter(8), const Duration(minutes: 15));
    expect(delayAfter(9), const Duration(hours: 1));
    expect(delayAfter(500), const Duration(hours: 1));
  });

  test('four wrong PINs are free, the fifth starts a 30 second delay', () async {
    final v = _Verifier(PinCheck.rejected);
    await failTimes(4, v);
    expect((await limiter.status()).isThrottled, isFalse);
    await failTimes(1, v);
    final s = await limiter.status();
    expect(s.failures, 5);
    expect(s.remaining, const Duration(seconds: 30));
  });

  test('while throttled the verifier is not called and nothing is counted',
      () async {
    final v = _Verifier(PinCheck.accepted);
    await failTimes(5, _Verifier(PinCheck.rejected));
    final r = await guard.check(v, '2468');
    expect(r, isA<GuardedThrottled>());
    expect(v.calls, 0);
    expect((await limiter.status()).failures, 5);
  });

  test('the delay passes with the monotonic clock and then grows', () async {
    final bad = _Verifier(PinCheck.rejected);
    await failTimes(5, bad);
    clock.advance(const Duration(seconds: 31));
    expect((await limiter.status()).isThrottled, isFalse);
    await failTimes(1, bad);
    expect((await limiter.status()).remaining, const Duration(minutes: 1));
  });

  test('a right PIN resets everything', () async {
    await failTimes(5, _Verifier(PinCheck.rejected));
    clock.advance(const Duration(seconds: 31));
    final r = await guard.check(_Verifier(PinCheck.accepted), '2468');
    expect((r as GuardedResult).check, PinCheck.accepted);
    final s = await limiter.status();
    expect(s.failures, 0);
    expect(s.isThrottled, isFalse);
  });

  test('a decoy PIN does not reset the count', () async {
    await failTimes(3, _Verifier(PinCheck.rejected));
    await guard.check(_Verifier(PinCheck.decoy), '0000');
    expect((await limiter.status()).failures, 3);
  });

  test('a too-short PIN and a failing check are not counted', () async {
    await failTimes(2, _Verifier(PinCheck.rejected));
    final v = _Verifier(PinCheck.rejected);
    final short = await guard.check(v, '12');
    expect((short as GuardedResult).check, PinCheck.tooShort);
    expect(v.calls, 0);
    v.failure = StateError('record unusable');
    await expectLater(guard.check(v, '9999'), throwsStateError);
    expect((await limiter.status()).failures, 2);
  });

  test('the count survives a restart', () async {
    await failTimes(5, _Verifier(PinCheck.rejected));
    final again = AttemptLimiter(store: store, clock: clock);
    final s = await again.status();
    expect(s.failures, 5);
    expect(s.isThrottled, isTrue);
  });

  test('killing the app mid-check still counts the attempt', () async {
    await limiter.begin(); // never followed by success or revert
    final again = AttemptLimiter(store: store, clock: clock);
    expect((await again.status()).failures, 1);
  });

  test('restarting the phone starts the full delay again', () async {
    await failTimes(5, _Verifier(PinCheck.rejected));
    clock.advance(const Duration(seconds: 29));
    clock
      ..boot = 2
      ..elapsedMs = 50; // elapsed time restarts after a reboot
    final s = await limiter.status();
    expect(s.remaining, const Duration(seconds: 30));
  });

  test('a stored delay longer than the policy allows is clamped', () async {
    await failTimes(5, _Verifier(PinCheck.rejected));
    await store.write(
      Uint8List.fromList(utf8.encode('v1|5|${clock.elapsedMs + 999999999}|1')),
    );
    expect((await limiter.status()).remaining, const Duration(seconds: 30));
  });

  test('a corrupt record fails closed but not forever', () async {
    await store.write(Uint8List.fromList(utf8.encode('garbage')));
    final s = await limiter.status();
    expect(s.failures, 9);
    expect(s.remaining, const Duration(hours: 1));
    clock.advance(const Duration(hours: 1, seconds: 1));
    expect((await limiter.status()).isThrottled, isFalse);
  });
}
