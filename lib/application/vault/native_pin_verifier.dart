import 'dart:typed_data';

import '../../domain/vault/vault_lock.dart';
import '../../platform/native/sargoshi_native.dart';

/// Supplies the 32-byte secret that keys every PIN record. On Android this is
/// a hardware-backed Keystore value that never leaves the device.
abstract interface class PepperSource {
  Future<Uint8List> pepper();
}

/// PIN check backed by the Rust core (Argon2id keyed with the pepper).
///
/// Both records are always evaluated so a decoy PIN costs the same time as a
/// real one. A record or pepper failure propagates as an exception: it must
/// never be shown as "wrong PIN" or silently open anything.
///
/// This runs the key derivation on the calling isolate. Wiring it to the UI
/// must move the work to a background isolate first.
final class NativePinVerifier {
  NativePinVerifier({
    required SargoshiNative native,
    required PepperSource pepperSource,
    required this.realRecord,
    this.decoyRecord,
  }) : _native = native,
       _pepperSource = pepperSource;

  final SargoshiNative _native;
  final PepperSource _pepperSource;
  final Uint8List realRecord;
  final Uint8List? decoyRecord;

  /// Creates the verifier record to store for [pin].
  static Future<Uint8List> enroll({
    required SargoshiNative native,
    required PepperSource pepperSource,
    required String pin,
  }) async => native.enrollVault(pin, await _pepperOf(pepperSource));

  static Future<Uint8List> _pepperOf(PepperSource source) async {
    final pepper = await source.pepper();
    if (pepper.length != 32) {
      throw const NativeCoreException(NativeCoreException.pepper);
    }
    return pepper;
  }

  Future<PinCheck> check(String pin) async {
    if (pin.length < kMinPinLength) return PinCheck.tooShort;
    final pepper = await _pepperOf(_pepperSource);
    final real = _native.verifyVault(realRecord, pin, pepper);
    final decoy = decoyRecord == null
        ? false
        : _native.verifyVault(decoyRecord!, pin, pepper);
    if (real) return PinCheck.accepted;
    if (decoy) return PinCheck.decoy;
    return PinCheck.rejected;
  }
}
