import 'dart:isolate';
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
final class NativePinVerifier implements PinVerifier {
  NativePinVerifier({
    required this.native,
    required this.pepperSource,
    required this.realRecord,
    this.decoyRecord,
  });

  final SargoshiNative native;
  final PepperSource pepperSource;
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

  @override
  Future<PinCheck> check(String pin) async {
    if (pin.length < kMinPinLength) return PinCheck.tooShort;
    final pepper = await _pepperOf(pepperSource);
    final real = native.verifyVault(realRecord, pin, pepper);
    final decoy = decoyRecord == null
        ? false
        : native.verifyVault(decoyRecord!, pin, pepper);
    if (real) return PinCheck.accepted;
    if (decoy) return PinCheck.decoy;
    return PinCheck.rejected;
  }
}

final class _FixedPepper implements PepperSource {
  const _FixedPepper(this.bytes);
  final Uint8List bytes;

  @override
  Future<Uint8List> pepper() async => bytes;
}

/// Same checks as [NativePinVerifier], but the key derivation runs on a
/// short-lived background isolate so the UI never freezes. Only plain bytes
/// and a path cross the isolate boundary.
final class IsolatePinVerifier implements PinVerifier {
  const IsolatePinVerifier({
    required this.libraryPath,
    required this.pepperSource,
    required this.realRecord,
    this.decoyRecord,
  });

  final String libraryPath;
  final PepperSource pepperSource;
  final Uint8List realRecord;
  final Uint8List? decoyRecord;

  @override
  Future<PinCheck> check(String pin) async {
    if (pin.length < kMinPinLength) return PinCheck.tooShort;
    final pepper = await NativePinVerifier._pepperOf(pepperSource);
    final path = libraryPath;
    final real = realRecord;
    final decoy = decoyRecord;
    return Isolate.run(
      () => NativePinVerifier(
        native: SargoshiNative.open(path),
        pepperSource: _FixedPepper(pepper),
        realRecord: real,
        decoyRecord: decoy,
      ).check(pin),
    );
  }
}
