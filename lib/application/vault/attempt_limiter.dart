import 'dart:convert';
import 'dart:typed_data';

import '../../domain/vault/attempt_policy.dart';
import '../../domain/vault/vault_lock.dart';

/// Persistent home for the attempt counter. Survives app restarts.
abstract interface class AttemptStore {
  Future<Uint8List?> read();
  Future<void> write(Uint8List bytes);
}

/// A reading of the device's monotonic clock. [elapsedMs] does not change when
/// the person edits the date or time. [boot] changes on every restart.
typedef AttemptTime = ({int elapsedMs, int boot});

abstract interface class AttemptClock {
  Future<AttemptTime> now();
}

/// Non-persistent store for desktop runs and tests.
final class InMemoryAttemptStore implements AttemptStore {
  Uint8List? _bytes;

  @override
  Future<Uint8List?> read() async => _bytes;

  @override
  Future<void> write(Uint8List bytes) async => _bytes = bytes;
}

/// Stopwatch clock for desktop runs and tests (one "boot" per process).
final class StopwatchAttemptClock implements AttemptClock {
  StopwatchAttemptClock() : _watch = Stopwatch()..start();

  final Stopwatch _watch;

  @override
  Future<AttemptTime> now() async =>
      (elapsedMs: _watch.elapsedMilliseconds, boot: 0);
}

final class AttemptStatus {
  const AttemptStatus({required this.failures, required this.remaining});

  final int failures;

  /// Time left before another attempt is allowed. Zero when allowed.
  final Duration remaining;

  bool get isThrottled => remaining > Duration.zero;
}

final class _State {
  const _State(this.failures, this.lockEndsMs, this.boot);

  final int failures;
  final int lockEndsMs;
  final int boot;

  static const fresh = _State(0, 0, 0);

  Uint8List encode() =>
      Uint8List.fromList(utf8.encode('v1|$failures|$lockEndsMs|$boot'));

  static _State? decode(Uint8List bytes) {
    final parts = utf8.decode(bytes, allowMalformed: true).split('|');
    if (parts.length != 4 || parts[0] != 'v1') return null;
    final f = int.tryParse(parts[1]);
    final l = int.tryParse(parts[2]);
    final b = int.tryParse(parts[3]);
    if (f == null || l == null || b == null || f < 0 || l < 0) return null;
    return _State(f, l, b);
  }
}

/// Counts wrong PINs across restarts and enforces the delays of
/// [delayAfter]. Counting happens BEFORE the PIN is checked and is undone only
/// for attempts that were not a wrong PIN, so killing the app mid-check does
/// not give a free guess. A restart of the phone does not shorten a delay: it
/// starts the full delay again. A corrupt record counts as many failures
/// rather than as none.
final class AttemptLimiter {
  AttemptLimiter({required this.store, required this.clock});

  final AttemptStore store;
  final AttemptClock clock;

  _State? _previous;

  Future<_State> _load(AttemptTime t) async {
    final bytes = await store.read();
    if (bytes == null || bytes.isEmpty) return _State.fresh;
    final state = _State.decode(bytes);
    if (state != null) return state;
    return _State(9, t.elapsedMs + delayAfter(9).inMilliseconds, t.boot);
  }

  _State _reconcile(_State s, AttemptTime t) {
    if (s.failures == 0) return s;
    final cap = t.elapsedMs + delayAfter(s.failures).inMilliseconds;
    if (s.boot != t.boot) return _State(s.failures, cap, t.boot);
    if (s.lockEndsMs > cap) return _State(s.failures, cap, s.boot);
    return s;
  }

  Duration _remaining(_State s, AttemptTime t) {
    final ms = s.lockEndsMs - t.elapsedMs;
    return ms > 0 ? Duration(milliseconds: ms) : Duration.zero;
  }

  Future<AttemptStatus> status() async {
    final t = await clock.now();
    final loaded = await _load(t);
    final state = _reconcile(loaded, t);
    if (state != loaded) await store.write(state.encode());
    return AttemptStatus(
      failures: state.failures,
      remaining: _remaining(state, t),
    );
  }

  /// Starts one attempt. Returns the remaining delay when none is allowed yet
  /// (nothing is counted), or null after counting the attempt.
  Future<Duration?> begin() async {
    final t = await clock.now();
    final state = _reconcile(await _load(t), t);
    final remaining = _remaining(state, t);
    if (remaining > Duration.zero) {
      await store.write(state.encode());
      return remaining;
    }
    _previous = state;
    final n = state.failures + 1;
    final next = _State(n, t.elapsedMs + delayAfter(n).inMilliseconds, t.boot);
    await store.write(next.encode());
    return null;
  }

  /// The PIN was right: forget all failures.
  Future<void> success() async {
    _previous = null;
    await store.write(_State.fresh.encode());
  }

  /// The attempt was not a wrong PIN (too short, check unavailable): give it
  /// back.
  Future<void> revert() async {
    final previous = _previous;
    _previous = null;
    if (previous != null) await store.write(previous.encode());
  }
}

/// Outcome of one guarded PIN check.
sealed class GuardedCheck {
  const GuardedCheck();
}

final class GuardedThrottled extends GuardedCheck {
  const GuardedThrottled(this.remaining);
  final Duration remaining;
}

final class GuardedResult extends GuardedCheck {
  const GuardedResult(this.check);
  final PinCheck check;
}

/// Every PIN check in the app goes through here, so the delays cannot be
/// bypassed by using a different screen.
final class VaultAttemptGuard {
  VaultAttemptGuard(this.limiter);

  final AttemptLimiter limiter;

  /// Throws what [verifier] throws, after giving the attempt back.
  Future<GuardedCheck> check(PinVerifier verifier, String pin) async {
    if (pin.length < kMinPinLength) {
      return const GuardedResult(PinCheck.tooShort);
    }
    final wait = await limiter.begin();
    if (wait != null) return GuardedThrottled(wait);
    final PinCheck result;
    try {
      result = await verifier.check(pin);
    } catch (_) {
      await limiter.revert();
      rethrow;
    }
    switch (result) {
      case PinCheck.accepted:
        await limiter.success();
      // A decoy PIN must NOT reset the count: whoever holds the decoy PIN
      // could otherwise reset it between guesses at the real one.
      case PinCheck.decoy || PinCheck.tooShort:
        await limiter.revert();
      case PinCheck.rejected:
        break;
    }
    return GuardedResult(result);
  }
}
