import 'dart:typed_data';

import '../domain/vault/vault_blob_store.dart';

/// Non-persistent blob store for tests and desktop demos. Data is lost when the
/// process ends, so it must never back a real vault.
final class InMemoryVaultBlobStore implements VaultBlobStore {
  final Map<String, Uint8List> _blobs = <String, Uint8List>{};

  @override
  Future<Uint8List?> read(String name) async {
    final value = _blobs[name];
    return value == null ? null : Uint8List.fromList(value);
  }

  @override
  Future<void> write(String name, Uint8List bytes) async {
    _blobs[name] = Uint8List.fromList(bytes);
  }

  @override
  Future<void> delete(String name) async {
    _blobs.remove(name);
  }

  @override
  Future<List<String>> list(String prefix) async =>
      _blobs.keys.where((name) => name.startsWith(prefix)).toList();
}
