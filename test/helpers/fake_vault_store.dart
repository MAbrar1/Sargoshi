import 'dart:typed_data';

import 'package:sargoshi/application/vault/vault_content.dart';
import 'package:sargoshi/application/vault/vault_session.dart';

/// In-memory [VaultItemStore] that records locking.
final class FakeVaultStore implements VaultItemStore {
  final Map<String, Uint8List> data = <String, Uint8List>{};
  bool locked = false;

  void _check() {
    if (locked) throw const VaultLockedException();
  }

  @override
  Future<void> put({
    required String kind,
    required String id,
    required Uint8List plain,
  }) async {
    _check();
    data['$kind/$id'] = Uint8List.fromList(plain);
  }

  @override
  Future<Uint8List> get({required String kind, required String id}) async {
    _check();
    final value = data['$kind/$id'];
    if (value == null) throw const VaultItemNotFoundException();
    return value;
  }

  @override
  Future<void> delete({required String kind, required String id}) async {
    _check();
    data.remove('$kind/$id');
  }

  @override
  Future<List<String>> list(String kind) async {
    _check();
    final prefix = '$kind/';
    return data.keys
        .where((k) => k.startsWith(prefix))
        .map((k) => k.substring(prefix.length))
        .toList()
      ..sort();
  }

  @override
  void lock() => locked = true;
}

/// Opener that hands out [FakeVaultStore]s and records how it was called.
final class FakeOpener implements VaultContentOpener {
  FakeOpener({this.fail = false, this.seed = const <String, Uint8List>{}});

  final bool fail;

  /// Items every opened store starts with, keyed `kind/id`.
  final Map<String, Uint8List> seed;
  final List<({String pin, bool decoy})> calls = [];
  final List<FakeVaultStore> stores = [];

  @override
  Future<VaultItemStore> open({
    required String pin,
    required bool decoy,
  }) async {
    calls.add((pin: pin, decoy: decoy));
    if (fail) throw StateError('content unavailable');
    final store = FakeVaultStore();
    store.data.addAll(seed);
    stores.add(store);
    return store;
  }
}
