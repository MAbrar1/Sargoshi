import 'dart:typed_data';

import 'package:flutter/services.dart' show MethodChannel, PlatformException;

import '../domain/vault/vault_blob_store.dart';

/// Raised when the platform cannot read or write a blob.
final class VaultBlobsUnavailableException implements Exception {
  const VaultBlobsUnavailableException();

  @override
  String toString() => 'VaultBlobsUnavailableException';
}

/// Blobs kept by `VaultBlobFiles.kt` in the no-backup directory.
final class ChannelVaultBlobStore implements VaultBlobStore {
  ChannelVaultBlobStore({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(channelName);

  static const channelName = 'app.sargoshi/vault_blobs';

  final MethodChannel _channel;

  Future<T?> _call<T>(String method, Map<String, Object?> args) async {
    try {
      return await _channel.invokeMethod<T>(method, args);
    } on PlatformException {
      throw const VaultBlobsUnavailableException();
    }
  }

  @override
  Future<Uint8List?> read(String name) =>
      _call<Uint8List>('read', <String, Object?>{'name': name});

  @override
  Future<void> write(String name, Uint8List bytes) =>
      _call<void>('write', <String, Object?>{'name': name, 'bytes': bytes});

  @override
  Future<void> delete(String name) =>
      _call<void>('delete', <String, Object?>{'name': name});

  @override
  Future<List<String>> list(String prefix) async {
    final names = await _call<List<Object?>>('list', <String, Object?>{
      'prefix': prefix,
    });
    return <String>[for (final name in names ?? const <Object?>[]) '$name'];
  }
}
