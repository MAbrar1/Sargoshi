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

/// A PIN change failed after the new PIN was already accepted by the verifier.
/// The vault stays readable (the next unlock recovers it), but the person must
/// be told the change may have taken effect.
final class VaultPinChangeIncompleteException implements Exception {
  const VaultPinChangeIncompleteException();

  @override
  String toString() => 'VaultPinChangeIncompleteException';
}

/// What the rest of the app may do with an unlocked vault. Implemented by
/// [VaultContent]; tests use in-memory fakes.
abstract interface class VaultItemStore {
  Future<void> put({
    required String kind,
    required String id,
    required Uint8List plain,
  });
  Future<Uint8List> get({required String kind, required String id});
  Future<void> delete({required String kind, required String id});
  Future<List<String>> list(String kind);

  /// Overwrites the in-memory key. Safe to call repeatedly.
  void lock();
}

/// Wraps a master key under a PIN and pepper. Slow (Argon2id).
typedef KeyWrapper =
    Future<Uint8List> Function(Uint8List master, String pin, Uint8List pepper);

/// Unwraps a master key. Throws [NativeCoreException] for a wrong PIN.
typedef KeyUnwrapper =
    Future<Uint8List> Function(Uint8List blob, String pin, Uint8List pepper);

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
final class VaultContent implements VaultItemStore {
  VaultContent({
    required this.native,
    required this.pepperSource,
    required this.blobs,
    this.namespace = 'real',
    KeyWrapper? wrapKey,
    KeyUnwrapper? unwrapKey,
  }) : _wrapKey = wrapKey ?? _directWrap(native),
       _unwrapKey = unwrapKey ?? _directUnwrap(native) {
    _checkName(namespace, _namespacePattern);
  }

  final SargoshiNative native;
  final PepperSource pepperSource;
  final VaultBlobStore blobs;
  final String namespace;
  final KeyWrapper _wrapKey;
  final KeyUnwrapper _unwrapKey;

  static KeyWrapper _directWrap(SargoshiNative native) =>
      (master, pin, pepper) async => native.wrapKey(master, pin, pepper);

  static KeyUnwrapper _directUnwrap(SargoshiNative native) =>
      (blob, pin, pepper) async => native.unwrapKey(blob, pin, pepper);

  static final RegExp _namespacePattern = RegExp(r'^[a-z0-9]{1,16}$');
  static final RegExp _kindPattern = RegExp(r'^[a-z0-9_]{1,32}$');
  static final RegExp _idPattern = RegExp(r'^[A-Za-z0-9_-]{1,64}$');
  static const int _masterLength = 32;

  Uint8List? _master;

  bool get isUnlocked => _master != null;

  String get _masterName => '$namespace/master';
  String get _nextName => '$namespace/masternext';
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
      await blobs.write(_masterName, await _wrapKey(master, pin, pepper));
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
    final pepper = await _pepper();
    Uint8List master;
    try {
      master = await _unwrapKey(blob, pin, pepper);
    } on NativeCoreException catch (e) {
      // A PIN change that stopped half-way leaves the new wrap beside the old
      // one. Try it, and finish the change when it opens.
      if (e.code != NativeCoreException.decrypt) rethrow;
      final next = await blobs.read(_nextName);
      if (next == null) rethrow;
      master = await _unwrapKey(next, pin, pepper);
      await _promote(next);
    }
    if (master.length != _masterLength) {
      master.fillRange(0, master.length, 0);
      throw const NativeCoreException(NativeCoreException.format);
    }
    _replaceMaster(master);
  }

  Future<void> _promote(Uint8List next) async {
    await blobs.write(_masterName, next);
    await blobs.delete(_nextName);
  }

  /// Re-wraps the master key under [newPin]; items stay as they are.
  ///
  /// [oldPin] is checked by unwrapping, so a wrong one throws
  /// [NativeCoreException] with [NativeCoreException.decrypt] and changes
  /// nothing. The order is crash-safe: the new wrap is stored beside the old
  /// one, then [commitVerifier] switches the PIN check to [newPin], then the
  /// new wrap replaces the old. An interruption at any point leaves one PIN
  /// working and [unlock] able to finish the change.
  ///
  /// Throws [VaultPinChangeIncompleteException] if a failure happens after
  /// [commitVerifier] succeeded. Leaves the vault unlocked on success.
  Future<void> changePin({
    required String oldPin,
    required String newPin,
    required Future<void> Function() commitVerifier,
  }) async {
    final blob = await blobs.read(_masterName);
    if (blob == null) throw const VaultMissingException();
    final pepper = await _pepper();
    final master = await _unwrapKey(blob, oldPin, pepper);
    if (master.length != _masterLength) {
      master.fillRange(0, master.length, 0);
      throw const NativeCoreException(NativeCoreException.format);
    }
    var committed = false;
    try {
      final next = await _wrapKey(master, newPin, pepper);
      await blobs.write(_nextName, next);
      await commitVerifier();
      committed = true;
      await _promote(next);
    } catch (_) {
      master.fillRange(0, master.length, 0);
      if (committed) throw const VaultPinChangeIncompleteException();
      // The verifier did not change, so the staged wrap is dead weight.
      try {
        await blobs.delete(_nextName);
      } catch (_) {}
      rethrow;
    }
    _replaceMaster(master);
  }

  /// Unlocks when a master key exists, otherwise creates one under [pin].
  /// Used after the PIN verifier has already accepted [pin].
  Future<void> unlockOrCreate(String pin) async {
    if (await exists()) {
      await unlock(pin);
    } else {
      await create(pin);
    }
  }

  void _replaceMaster(Uint8List master) {
    lock();
    _master = master;
  }

  /// Overwrites the in-memory master key. Safe to call repeatedly.
  @override
  void lock() {
    final key = _master;
    if (key != null) key.fillRange(0, key.length, 0);
    _master = null;
  }

  /// Stores [plain] under [kind] and [id], replacing any previous value.
  @override
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
  @override
  Future<Uint8List> get({required String kind, required String id}) async {
    _checkName(kind, _kindPattern);
    _checkName(id, _idPattern);
    final key = _key();
    final sealed = await blobs.read(_itemName(kind, id));
    if (sealed == null) throw const VaultItemNotFoundException();
    return native.open(key, _aad(kind, id), sealed);
  }

  @override
  Future<void> delete({required String kind, required String id}) async {
    _checkName(kind, _kindPattern);
    _checkName(id, _idPattern);
    _key();
    await blobs.delete(_itemName(kind, id));
  }

  /// Ids of stored items of [kind], sorted. Needs the vault unlocked so a
  /// locked vault does not reveal what it holds.
  @override
  Future<List<String>> list(String kind) async {
    _checkName(kind, _kindPattern);
    _key();
    final prefix = '$namespace/item/$kind/';
    final names = await blobs.list(prefix);
    return names.map((n) => n.substring(prefix.length)).toList()..sort();
  }
}
