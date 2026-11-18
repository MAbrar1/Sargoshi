import 'dart:typed_data';

import 'package:flutter/services.dart' show MethodChannel, PlatformException;

import '../application/vault/native_pin_verifier.dart';

/// Raised when the platform cannot produce the vault secret. Callers treat it
/// as "vault unavailable", never as a wrong PIN.
final class PepperUnavailableException implements Exception {
  const PepperUnavailableException();

  @override
  String toString() => 'PepperUnavailableException';
}

/// Vault secret held by the Android Keystore (see `VaultPepperStore.kt`).
final class KeystorePepperSource implements PepperSource {
  KeystorePepperSource({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(channelName);

  static const channelName = 'app.sargoshi/vault_pepper';

  final MethodChannel _channel;

  /// Reads the existing secret. Never creates one, so records written with a
  /// lost secret cannot silently turn into "wrong PIN".
  @override
  Future<Uint8List> pepper() => _bytes('read');

  /// Creates the secret on first enrollment and returns it.
  Future<Uint8List> createIfMissing() => _bytes('createIfMissing');

  Future<bool> exists() async {
    try {
      return await _channel.invokeMethod<bool>('exists') ?? false;
    } on PlatformException {
      throw const PepperUnavailableException();
    }
  }

  Future<Uint8List> _bytes(String method) async {
    try {
      final bytes = await _channel.invokeMethod<Uint8List>(method);
      if (bytes == null || bytes.length != 32) {
        throw const PepperUnavailableException();
      }
      return bytes;
    } on PlatformException {
      throw const PepperUnavailableException();
    }
  }
}
