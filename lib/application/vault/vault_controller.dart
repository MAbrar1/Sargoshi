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
  PinCheck check(String pin) {
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
  @override
  VaultLock build() => const VaultLocked();

  VaultAttemptOutcome submit(String pin) {
    final result = submitPin(state, ref.read(pinVerifierProvider).check(pin));
    state = result.state;
    return result.outcome;
  }

  void lock() => state = lockVault(state);
}

final vaultControllerProvider = NotifierProvider<VaultController, VaultLock>(
  VaultController.new,
);
