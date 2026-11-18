import 'dart:typed_data';

/// Persistent home for opaque, already-encrypted vault blobs (the wrapped
/// master key and sealed items). Implementations must write atomically and keep
/// the data out of backups. Names are validated by the caller.
abstract interface class VaultBlobStore {
  Future<Uint8List?> read(String name);
  Future<void> write(String name, Uint8List bytes);
  Future<void> delete(String name);

  /// Names that start with [prefix], in no particular order.
  Future<List<String>> list(String prefix);
}
