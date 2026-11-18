import 'dart:isolate';
import 'dart:typed_data';

import '../../domain/vault/vault_lock.dart';
import '../../domain/vault/vault_records.dart';
import '../../platform/native/sargoshi_native.dart';
import 'native_pin_verifier.dart';

/// Creates the vault secret the first time a PIN is set.
abstract interface class PepperProvisioner {
  Future<Uint8List> createIfMissing();
}

enum VaultSetupError { tooShort, decoySameAsReal }

final class VaultSetupException implements Exception {
  const VaultSetupException(this.error);

  final VaultSetupError error;

  @override
  String toString() => 'VaultSetupException($error)';
}

/// Sets, replaces or removes the vault PINs. Key derivation runs on a
/// background isolate.
final class VaultSetup {
  const VaultSetup({
    required this.libraryPath,
    required this.provisioner,
    required this.records,
  });

  final String libraryPath;
  final PepperProvisioner provisioner;
  final VaultRecordStore records;

  Future<bool> isSetUp() async => await records.read(VaultSlot.real) != null;

  /// Stores a record for [pin], and for [decoyPin] when given. The decoy is
  /// removed when it is null. Nothing is written if validation fails.
  Future<void> setPins({required String pin, String? decoyPin}) async {
    if (pin.length < kMinPinLength ||
        (decoyPin != null && decoyPin.length < kMinPinLength)) {
      throw const VaultSetupException(VaultSetupError.tooShort);
    }
    if (decoyPin != null && decoyPin == pin) {
      throw const VaultSetupException(VaultSetupError.decoySameAsReal);
    }
    final pepper = await provisioner.createIfMissing();
    final path = libraryPath;
    final real = await Isolate.run(
      () => SargoshiNative.open(path).enrollVault(pin, pepper),
    );
    final decoy = decoyPin == null
        ? null
        : await Isolate.run(
            () => SargoshiNative.open(path).enrollVault(decoyPin, pepper),
          );
    // The decoy goes first so a half-finished change never leaves the old
    // decoy next to a new real PIN that equals it.
    if (decoy == null) {
      await records.delete(VaultSlot.decoy);
    } else {
      await records.write(VaultSlot.decoy, decoy);
    }
    await records.write(VaultSlot.real, real);
  }
}

/// Verifies PINs against the stored records. Never reports a missing or
/// unreadable record as a wrong PIN.
final class StoredPinVerifier implements PinVerifier {
  const StoredPinVerifier({
    required this.libraryPath,
    required this.pepperSource,
    required this.records,
  });

  final String libraryPath;
  final PepperSource pepperSource;
  final VaultRecordStore records;

  @override
  Future<PinCheck> check(String pin) async {
    final real = await records.read(VaultSlot.real);
    if (real == null) throw const VaultNotSetUpException();
    final decoy = await records.read(VaultSlot.decoy);
    return IsolatePinVerifier(
      libraryPath: libraryPath,
      pepperSource: pepperSource,
      realRecord: real,
      decoyRecord: decoy,
    ).check(pin);
  }
}
