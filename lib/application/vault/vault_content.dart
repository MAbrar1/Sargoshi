import 'dart:convert';
import 'dart:typed_data';

import '../../domain/vault/vault_blob_store.dart';
import '../../platform/native/sargoshi_native.dart';
import 'native_pin_verifier.dart' show PepperSource;

/// The vault is locked, so no content can be read or written.
final class VaultLockedException implements Exception {
  const VaultLockedException();

  @override
  String toString() => 'VaultLockedException';
}

/// [VaultContent.create] was called on a store that already has a master key.
final class VaultAlreadyExistsException implements Exception {
  const VaultAlreadyExistsException();

  @override
  String toString() => 'VaultAlreadyExistsException';
}

/// [VaultContent.unlock] found no master key. Not the same as a wrong PIN.
final class VaultMissingException implements Exception {
  const VaultMissingException();

  @override
  String toString() => 'VaultMissingException';
}

/// No item exists under that kind and id.
final class VaultItemNotFoundException implements Exception {
  const VaultItemNotFoundException();

  @override
  String toString() => 'VaultItemNotFoundException';
}

/// A kind, id or namespace contained characters outside the allowed set.
final class VaultNameException implements Exception {
  const VaultNameException();

  @override
  String toString() => 'VaultNameException';
}

/// Encrypted content store for one vault namespace (real or decoy).
///
/// A random 32-byte master key is wrapped under the PIN and the platform
/// pepper (Argon2id, in the Rust core) and stored as one blob. Every item is
/// sealed under the master key with its namespace, kind and id bound as
/// authenticated context, so a blob copied to another slot fails to open.
///
/// Only ciphertext reaches the [VaultBlobStore]. The master key exists in
/// memory only between [unlock]/[create] and [lock]; [lock] overwrites it.
/// Dart cannot guarantee no other copy exists, which is a documented residual
/// risk (see the threat model).
///
/// [create] and [unlock] run Argon2id on the calling isolate: call them from a
/// background isolate or show a busy state.
final class VaultContent {
  VaultContent({
    required this.native,
    required this.pepperSource,
    required this.blobs,
    this.namespace = 'real',
  }) {
    _checkName(namespace, _namespacePattern);
  }

  final SargoshiNative native;
  final PepperSource pepperSource;
  final VaultBlobStore blobs;
  final String namespace;

  static final RegExp _namespacePattern = RegExp(r'^[a-z0-9]{1,16}$');
  static final RegExp _kindPattern = RegExp(r'^[a-z0-9_]{1,32}$');
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9_-]{1,64}$');
  static const int _masterLength = 32;

  Uint8List? _master;

  bool get isUnlocked => _master != null;

  String get _masterName => '$namespace/master';
  String _itemName(String kind, String id) => '$namespace/item/$kind/$id';

  static void _checkName(String value, RegExp pattern) {
    if (!pattern.hasMatch(value)) throw const VaultNameException();
  }

  Future<Uint8List> _pepper() async {
    final pepper = await pepperSource.pepper();
    if (pepper.length != 32) {
      throw const NativeCoreException(NativeCoreException.pepper);
    }
    return pepper;
  }

  Uint8List _aad(String kind, String id) => Uint8List.fromList(
    utf8.encode('sargoshi.vault.item.v1|$namespace|$kind|$id'),
  );

  Uint8List _key() {
    final key = _master;
    if (key == null) throw const VaultLockedException();
    return key;
  }

  /// True once a master key has been stored for this namespace.
  Future<bool> exists() async => (await blobs.read(_masterName)) != null;

  /// Creates a new master key wrapped under [pin] and leaves the vault
  /// unlocked. Throws [VaultAlreadyExistsException] rather than overwrite.
  Future<void> create(String pin) async {
    if (await exists()) throw const VaultAlreadyExistsException();
    final pepper = await _pepper();
    final master = native.generateKey();
    try {
      await blobs.write(_masterName, native.wrapKey(master, pin, pepper));
    } catch (_) {
      master.fillRange(0, master.length, 0);
      rethrow;
    }
    _replaceMaster(master);
  }

  /// Unlocks with [pin]. A wrong PIN, wrong pepper or tampered blob throws
  /// [NativeCoreException] with [NativeCoreException.decrypt] and leaves the
  /// vault locked.
  Future<void> unlock(String pin) async {
    final blob = await blobs.read(_masterName);
    if (blob == null) throw const VaultMissingException();
    final master = native.unwrapKey(blob, pin, await _pepper());
    if (master.length != _masterLength) {
      master.fillRange(0, master.length, 0);
      throw const NativeCoreException(NativeCoreException.format);
    }
    _replaceMaster(master);
  }

  void _replaceMaster(Uint8List master) {
    lock();
    _master = master;
  }

  /// Overwrites the in-memory master key. Safe to call repeatedly.
  void lock() {
    final key = _master;
    if (key != null) key.fillRange(0, key.length, 0);
    _master = null;
  }

  /// Stores [plain] under [kind] and [id], replacing any previous value.
  Future<void> put({
    required String kind,
    required String id,
    required Uint8List plain,
  }) async {
    _checkName(kind, _kindPattern);
    _checkName(id, _idPattern);
    final sealed = native.seal(_key(), _aad(kind, id), plain);
    await blobs.write(_itemName(kind, id), sealed);
  }

  /// Reads an item. Throws [VaultItemNotFoundException] when absent and
  /// [NativeCoreException] with [NativeCoreException.open] when the blob was
  /// tampered with or moved to another slot.
  Future<Uint8List> get({required String kind, required String id}) async {
    _checkName(kind, _kindPattern);
    _checkName(id, _idPattern);
    final key = _key();
    final sealed = await blobs.read(_itemName(kind, id));
    if (sealed == null) throw const VaultItemNotFoundException();
    return native.open(key, _aad(kind, id), sealed);
  }

  Future<void> delete({required String kind, required String id}) async {
    _checkName(kind, _kindPattern);
    _checkName(id, _idPattern);
    _key();
    await blobs.delete(_itemName(kind, id));
  }

  /// Ids of stored items of [kind], sorted. Needs the vault unlocked so a
  /// locked vault does not reveal what it holds.
  Future<List<String>> list(String kind) async {
    _checkName(kind, _kindPattern);
    _key();
    final prefix = '$namespace/item/$kind/';
    final names = await blobs.list(prefix);
    return names.map((n) => n.substring(prefix.length)).toList()..sort();
  }
}
