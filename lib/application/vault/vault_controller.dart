import 'dart:io' show Platform;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/vault/vault_lock.dart';
import '../../platform/attempt_channels.dart';
import 'attempt_limiter.dart';
import 'vault_backend.dart';
import 'vault_session.dart';

/// Stand-in verifier with fixed PINs and nothing stored. It protects nothing.
/// Stage 2 replaces it with a Keystore-bound check (Argon2id plus a
/// Keystore secret, spec section 4).
final class DemoPinVerifier implements PinVerifier {
  const DemoPinVerifier();

  static const demoPin = '1234';
  static const demoDecoyPin = '0000';

  @override
  Future<PinCheck> check(String pin) async {
    if (pin.length < kMinPinLength) return PinCheck.tooShort;
    if (pin == demoPin) return PinCheck.accepted;
    if (pin == demoDecoyPin) return PinCheck.decoy;
    return PinCheck.rejected;
  }
}

final pinVerifierProvider = Provider<PinVerifier>(
  (ref) => ref.watch(vaultBackendProvider)?.verifier ?? const DemoPinVerifier(),
);

/// Counts wrong PINs across restarts. Android keeps the count in the vault
/// record files and uses the monotonic clock; elsewhere it lives in memory.
final attemptLimiterProvider = Provider<AttemptLimiter>(
  (ref) => Platform.isAndroid
      ? AttemptLimiter(
          store: ChannelAttemptStore(),
          clock: ChannelAttemptClock(),
        )
      : AttemptLimiter(
          store: InMemoryAttemptStore(),
          clock: StopwatchAttemptClock(),
        ),
);

/// All PIN checks go through this guard, on every screen.
final attemptGuardProvider = Provider<VaultAttemptGuard>(
  (ref) => VaultAttemptGuard(ref.watch(attemptLimiterProvider)),
);

/// Failures so far and the delay left before the next attempt.
final attemptStatusProvider = FutureProvider.autoDispose<AttemptStatus>(
  (ref) => ref.watch(attemptLimiterProvider).status(),
);

final class VaultController extends Notifier<VaultLock> {
  bool _busy = false;
  int _generation = 0;

  @override
  VaultLock build() => const VaultLocked();

  /// Runs one attempt. Overlapping attempts are ignored, and a result that
  /// arrives after the vault was locked again is discarded.
  Future<VaultAttemptOutcome> submit(String pin) async {
    if (_busy) return VaultAttemptOutcome.ignored;
    _busy = true;
    final generation = _generation;
    try {
      final guarded = await ref
          .read(attemptGuardProvider)
          .check(ref.read(pinVerifierProvider), pin);
      if (generation != _generation) return VaultAttemptOutcome.ignored;
      if (guarded is GuardedThrottled) return VaultAttemptOutcome.throttled;
      final check = (guarded as GuardedResult).check;
      final opener = ref.read(vaultContentOpenerProvider);
      if (opener != null &&
          (check == PinCheck.accepted || check == PinCheck.decoy)) {
        // If the content cannot be opened, the vault stays locked.
        final store = await opener.open(
          pin: pin,
          decoy: check == PinCheck.decoy,
        );
        if (generation != _generation) {
          store.lock();
          return VaultAttemptOutcome.ignored;
        }
        ref.read(vaultContentSessionProvider.notifier).open(store);
      }
      final result = submitPin(state, check);
      state = result.state;
      return result.outcome;
    } on Object {
      return VaultAttemptOutcome.unavailable;
    } finally {
      _busy = false;
      ref.invalidate(attemptStatusProvider);
    }
  }

  void lock() {
    _generation++;
    ref.read(vaultContentSessionProvider.notifier).close();
    state = lockVault(state);
  }
}

final vaultControllerProvider = NotifierProvider<VaultController, VaultLock>(
  VaultController.new,
);
