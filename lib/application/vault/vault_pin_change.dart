import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/vault/vault_lock.dart';
import '../../platform/native/sargoshi_native.dart';
import 'vault_backend.dart';
import 'vault_content.dart';
import 'vault_controller.dart';
import 'vault_session.dart';
import 'vault_setup.dart';

enum PinChangeResult {
  changed,
  wrongCurrent,
  tooShort,
  sameAsCurrent,
  clashesWithDecoy,

  /// The new PIN took effect but finishing the change failed. The vault can
  /// still be opened; the person must be told.
  incomplete,

  /// Nothing was changed.
  failed,
}

/// Changes the real vault PIN: checks the current PIN, re-wraps the master key
/// and switches the PIN check, in the crash-safe order of
/// [VaultContent.changePin]. Only the real vault may change its PIN.
final class VaultPinChange {
  const VaultPinChange(this._ref);

  final Ref _ref;

  Future<PinChangeResult> change({
    required String currentPin,
    required String newPin,
  }) async {
    if (newPin.length < kMinPinLength) return PinChangeResult.tooShort;
    if (newPin == currentPin) return PinChangeResult.sameAsCurrent;
    final backend = _ref.read(vaultBackendProvider);
    final opener = _ref.read(vaultContentOpenerProvider);
    if (backend == null ||
        opener == null ||
        _ref.read(vaultControllerProvider) is! VaultOpen) {
      return PinChangeResult.failed;
    }
    try {
      if (await backend.verifier.check(currentPin) != PinCheck.accepted) {
        return PinChangeResult.wrongCurrent;
      }
      final store = await opener.changePin(
        oldPin: currentPin,
        newPin: newPin,
        commitVerifier: () => backend.changeRealPin(newPin),
      );
      _ref.read(vaultContentSessionProvider.notifier).open(store);
      return PinChangeResult.changed;
    } on VaultSetupException catch (e) {
      return e.error == VaultSetupError.decoySameAsReal
          ? PinChangeResult.clashesWithDecoy
          : PinChangeResult.tooShort;
    } on VaultPinChangeIncompleteException {
      return PinChangeResult.incomplete;
    } on NativeCoreException catch (e) {
      return e.code == NativeCoreException.decrypt
          ? PinChangeResult.wrongCurrent
          : PinChangeResult.failed;
    } on Object {
      return PinChangeResult.failed;
    }
  }
}

final vaultPinChangeProvider = Provider<VaultPinChange>(VaultPinChange.new);
