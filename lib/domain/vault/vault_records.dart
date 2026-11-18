import 'dart:typed_data';

/// Which PIN a stored verifier record belongs to.
enum VaultSlot { real, decoy }

/// Persistent home for PIN verifier records. Implementations must be atomic
/// per record and keep the data out of backups.
abstract interface class VaultRecordStore {
  Future<Uint8List?> read(VaultSlot slot);
  Future<void> write(VaultSlot slot, Uint8List record);
  Future<void> delete(VaultSlot slot);
}

/// Raised when the vault is used before a PIN was set. Not a wrong PIN.
final class VaultNotSetUpException implements Exception {
  const VaultNotSetUpException();

  @override
  String toString() => 'VaultNotSetUpException';
}
