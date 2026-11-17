import 'dart:typed_data';

/// A sealed message ready for transport. Opaque to everything but the core.
final class SealedMessage {
  const SealedMessage(this.bytes);
  final Uint8List bytes;
}

/// Raised when a sealed message cannot be opened. Deliberately carries no
/// detail on why: wrong key, tampering and truncation look the same.
final class CryptoOpenException implements Exception {
  const CryptoOpenException();
  @override
  String toString() => 'CryptoOpenException';
}

/// Boundary between the app and all cryptography (spec section 6). UI and
/// application code depend on this interface only. The real implementation
/// will call the Rust core over FFI.
abstract interface class CryptoCore {
  /// Stable name, shown in diagnostics.
  String get implementationId;

  /// False for any implementation that must never ship in a release build.
  bool get isProductionGrade;

  Future<SealedMessage> seal({
    required String contactId,
    required Uint8List plaintext,
  });

  /// Throws [CryptoOpenException] on any failure.
  Future<Uint8List> open({
    required String contactId,
    required SealedMessage message,
  });
}
