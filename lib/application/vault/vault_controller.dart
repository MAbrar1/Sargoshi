import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/vault/vault_lock.dart';

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
  (ref) => const DemoPinVerifier(),
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
      final check = await ref.read(pinVerifierProvider).check(pin);
      if (generation != _generation) return VaultAttemptOutcome.ignored;
      final result = submitPin(state, check);
      state = result.state;
      return result.outcome;
    } on Object {
      return VaultAttemptOutcome.unavailable;
    } finally {
      _busy = false;
    }
  }

  void lock() {
    _generation++;
    state = lockVault(state);
  }
}

final vaultControllerProvider = NotifierProvider<VaultController, VaultLock>(
  VaultController.new,
);
