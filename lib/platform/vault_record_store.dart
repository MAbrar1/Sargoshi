import 'dart:typed_data';

import 'package:flutter/services.dart' show MethodChannel, PlatformException;

import '../domain/vault/vault_records.dart';

/// Raised when the platform cannot read or write a record.
final class VaultRecordsUnavailableException implements Exception {
  const VaultRecordsUnavailableException();

  @override
  String toString() => 'VaultRecordsUnavailableException';
}

/// Records kept by `VaultRecordFiles.kt` in the no-backup directory.
final class ChannelVaultRecordStore implements VaultRecordStore {
  ChannelVaultRecordStore({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(channelName);

  static const channelName = 'app.sargoshi/vault_records';

  final MethodChannel _channel;

  Future<T?> _call<T>(String method, VaultSlot slot, [Uint8List? bytes]) async {
    try {
      return await _channel.invokeMethod<T>(method, <String, Object?>{
        'name': slot.name,
        'bytes': ?bytes,
      });
    } on PlatformException {
      throw const VaultRecordsUnavailableException();
    }
  }

  @override
  Future<Uint8List?> read(VaultSlot slot) => _call<Uint8List>('read', slot);

  @override
  Future<void> write(VaultSlot slot, Uint8List record) =>
      _call<void>('write', slot, record);

  @override
  Future<void> delete(VaultSlot slot) => _call<void>('delete', slot);
}
