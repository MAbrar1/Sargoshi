import 'dart:typed_data';

import '../domain/crypto/crypto_core.dart';

/// TEST AND DEMO ONLY. This performs NO encryption: it prefixes the plaintext
/// with a marker so tests can exercise the interface. It protects nothing.
/// `tool/ci/check_release_ready.py` blocks release builds while this is the
/// only [CryptoCore] and limits which files may import it.
final class FakeCryptoCore implements CryptoCore {
  const FakeCryptoCore();

  static const _marker = <int>[0x46, 0x41, 0x4b, 0x45]; // "FAKE"

  @override
  String get implementationId => 'fake-no-encryption';

  @override
  bool get isProductionGrade => false;

  @override
  Future<SealedMessage> seal({
    required String contactId,
    required Uint8List plaintext,
  }) async => SealedMessage(Uint8List.fromList([..._marker, ...plaintext]));

  @override
  Future<Uint8List> open({
    required String contactId,
    required SealedMessage message,
  }) async {
    final b = message.bytes;
    if (b.length < _marker.length) throw const CryptoOpenException();
    for (var i = 0; i < _marker.length; i++) {
      if (b[i] != _marker[i]) throw const CryptoOpenException();
    }
    return Uint8List.sublistView(b, _marker.length);
  }
}
