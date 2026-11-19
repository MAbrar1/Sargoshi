import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/vault/native_pin_verifier.dart';
import 'package:sargoshi/application/vault/vault_content.dart';
import 'package:sargoshi/domain/vault/vault_blob_store.dart';
import 'package:sargoshi/platform/native/sargoshi_native.dart';
import 'package:sargoshi/testing/in_memory_vault_blob_store.dart';

final class _Pepper implements PepperSource {
  final bytes = Uint8List.fromList(List.filled(32, 5));
  @override
  Future<Uint8List> pepper() async => bytes;
}

/// Blob store that can be told to fail writes to chosen names.
final class _FlakyBlobs implements VaultBlobStore {
  final inner = InMemoryVaultBlobStore();
  final failWrites = <String>{};

  @override
  Future<Uint8List?> read(String name) => inner.read(name);

  @override
  Future<void> write(String name, Uint8List bytes) async {
    if (failWrites.contains(name)) throw StateError('disk full');
    await inner.write(name, bytes);
  }

  @override
  Future<void> delete(String name) => inner.delete(name);

  @override
  Future<List<String>> list(String prefix) => inner.list(prefix);
}

Matcher _decrypt() => throwsA(
  isA<NativeCoreException>().having(
    (e) => e.code,
    'code',
    NativeCoreException.decrypt,
  ),
);

/// Runs against the host build of `sargoshi_ffi`. CI sets SARGOSHI_FFI_LIB.
void main() {
  final path = Platform.environment['SARGOSHI_FFI_LIB'];
  final skip = path == null
      ? 'SARGOSHI_FFI_LIB not set; build rust/sargoshi_ffi to run'
      : false;

  group('vault PIN change', skip: skip, () {
    late SargoshiNative native;
    late _FlakyBlobs blobs;
    final note = Uint8List.fromList([1, 2, 3, 4]);

    setUp(() {
      native = SargoshiNative.open(path!);
      blobs = _FlakyBlobs();
    });

    VaultContent vault() =>
        VaultContent(native: native, pepperSource: _Pepper(), blobs: blobs);

    Future<VaultContent> created() async {
      final v = vault();
      await v.create('2468');
      await v.put(kind: 'note', id: 'a1', plain: note);
      return v;
    }

    test('items stay readable and only the new PIN unlocks', () async {
      final v = await created();
      var committed = 0;
      await v.changePin(
        oldPin: '2468',
        newPin: '8642',
        commitVerifier: () async => committed++,
      );
      expect(committed, 1);
      expect(v.isUnlocked, isTrue);
      expect(await v.get(kind: 'note', id: 'a1'), note);
      expect(await blobs.read('real/masternext'), isNull);

      final fresh = vault();
      await expectLater(fresh.unlock('2468'), _decrypt());
      await fresh.unlock('8642');
      expect(await fresh.get(kind: 'note', id: 'a1'), note);
    });

    test('a wrong current PIN changes nothing', () async {
      final v = await created();
      var committed = 0;
      await expectLater(
        v.changePin(
          oldPin: '1111',
          newPin: '8642',
          commitVerifier: () async => committed++,
        ),
        _decrypt(),
      );
      expect(committed, 0);
      expect(await blobs.read('real/masternext'), isNull);
      await vault().unlock('2468');
    });

    test('if the verifier cannot switch, the old PIN keeps working', () async {
      final v = await created();
      await expectLater(
        v.changePin(
          oldPin: '2468',
          newPin: '8642',
          commitVerifier: () async => throw StateError('records down'),
        ),
        throwsStateError,
      );
      expect(await blobs.read('real/masternext'), isNull);
      final fresh = vault();
      await fresh.unlock('2468');
      expect(await fresh.get(kind: 'note', id: 'a1'), note);
      await expectLater(vault().unlock('8642'), _decrypt());
    });

    test('a failure after the verifier switched is reported and the next '
        'unlock finishes the change', () async {
      final v = await created();
      blobs.failWrites.add('real/master');
      await expectLater(
        v.changePin(
          oldPin: '2468',
          newPin: '8642',
          commitVerifier: () async {},
        ),
        throwsA(isA<VaultPinChangeIncompleteException>()),
      );
      expect(await blobs.read('real/masternext'), isNotNull);

      blobs.failWrites.clear();
      final fresh = vault();
      await fresh.unlock('8642');
      expect(await fresh.get(kind: 'note', id: 'a1'), note);
      expect(await blobs.read('real/masternext'), isNull);
      await expectLater(vault().unlock('2468'), _decrypt());
    });

    test('a wrong PIN still fails while a staged change exists', () async {
      final v = await created();
      blobs.failWrites.add('real/master');
      await expectLater(
        v.changePin(
          oldPin: '2468',
          newPin: '8642',
          commitVerifier: () async {},
        ),
        throwsA(isA<VaultPinChangeIncompleteException>()),
      );
      blobs.failWrites.clear();
      await expectLater(vault().unlock('9999'), _decrypt());
      expect(await blobs.read('real/masternext'), isNotNull);
    });

    test('changing before any vault exists is "missing", not a crash', () async {
      await expectLater(
        vault().changePin(
          oldPin: '2468',
          newPin: '8642',
          commitVerifier: () async {},
        ),
        throwsA(isA<VaultMissingException>()),
      );
    });
  });
}
