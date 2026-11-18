import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/vault/native_pin_verifier.dart';
import 'package:sargoshi/application/vault/vault_content.dart';
import 'package:sargoshi/platform/native/sargoshi_native.dart';
import 'package:sargoshi/testing/in_memory_vault_blob_store.dart';

final class _Pepper implements PepperSource {
  _Pepper(int fill) : bytes = Uint8List.fromList(List.filled(32, fill));
  final Uint8List bytes;
  @override
  Future<Uint8List> pepper() async => bytes;
}

Matcher _code(int code) =>
    throwsA(isA<NativeCoreException>().having((e) => e.code, 'code', code));

/// Runs against the host build of `sargoshi_ffi`. CI sets SARGOSHI_FFI_LIB.
void main() {
  final path = Platform.environment['SARGOSHI_FFI_LIB'];
  final skip = path == null
      ? 'SARGOSHI_FFI_LIB not set; build rust/sargoshi_ffi to run'
      : false;

  group('vault content', skip: skip, () {
    late SargoshiNative native;
    late InMemoryVaultBlobStore blobs;
    final pepper = _Pepper(5);

    setUp(() {
      native = SargoshiNative.open(path!);
      blobs = InMemoryVaultBlobStore();
    });

    VaultContent vault({String ns = 'real', PepperSource? source}) =>
        VaultContent(
          native: native,
          pepperSource: source ?? pepper,
          blobs: blobs,
          namespace: ns,
        );

    final note = Uint8List.fromList('نرمی سے کہو'.codeUnits);

    test('create unlocks, stores and reads items back', () async {
      final v = vault();
      expect(await v.exists(), isFalse);
      await v.create('2468');
      expect(v.isUnlocked, isTrue);
      expect(await v.exists(), isTrue);
      await v.put(kind: 'note', id: 'a1', plain: note);
      expect(await v.get(kind: 'note', id: 'a1'), note);
      expect(await v.list('note'), ['a1']);
    });

    test('items are not stored in plain', () async {
      final v = vault();
      await v.create('2468');
      await v.put(kind: 'note', id: 'a1', plain: note);
      final stored = await blobs.read('real/item/note/a1');
      expect(stored, isNotNull);
      expect(stored!.length, greaterThan(note.length));
      final text = String.fromCharCodes(stored);
      expect(text.contains(String.fromCharCodes(note)), isFalse);
    });

    test('a new instance unlocks with the PIN and reads old items', () async {
      final first = vault();
      await first.create('2468');
      await first.put(kind: 'note', id: 'a1', plain: note);
      first.lock();
      final second = vault();
      await second.unlock('2468');
      expect(await second.get(kind: 'note', id: 'a1'), note);
    });

    test('locked vault refuses every content call', () async {
      final v = vault();
      await v.create('2468');
      await v.put(kind: 'note', id: 'a1', plain: note);
      v.lock();
      expect(v.isUnlocked, isFalse);
      expect(
        () => v.get(kind: 'note', id: 'a1'),
        throwsA(isA<VaultLockedException>()),
      );
      expect(
        () => v.put(kind: 'note', id: 'a2', plain: note),
        throwsA(isA<VaultLockedException>()),
      );
      expect(() => v.list('note'), throwsA(isA<VaultLockedException>()));
      expect(
        () => v.delete(kind: 'note', id: 'a1'),
        throwsA(isA<VaultLockedException>()),
      );
    });

    test('wrong PIN or wrong pepper fails and stays locked', () async {
      final first = vault();
      await first.create('2468');
      first.lock();
      final wrongPin = vault();
      await expectLater(
        wrongPin.unlock('9999'),
        _code(NativeCoreException.decrypt),
      );
      expect(wrongPin.isUnlocked, isFalse);
      final wrongPepper = vault(source: _Pepper(6));
      await expectLater(
        wrongPepper.unlock('2468'),
        _code(NativeCoreException.decrypt),
      );
      expect(wrongPepper.isUnlocked, isFalse);
    });

    test('unlock without a vault is not a wrong PIN', () async {
      await expectLater(
        vault().unlock('2468'),
        throwsA(isA<VaultMissingException>()),
      );
    });

    test('create never overwrites an existing vault', () async {
      await vault().create('2468');
      await expectLater(
        vault().create('1357'),
        throwsA(isA<VaultAlreadyExistsException>()),
      );
      await vault().unlock('2468');
    });

    test('a blob moved to another slot does not open', () async {
      final v = vault();
      await v.create('2468');
      await v.put(kind: 'note', id: 'a1', plain: note);
      final original = (await blobs.read('real/item/note/a1'))!;
      await blobs.write('real/item/note/b2', original);
      await expectLater(
        v.get(kind: 'note', id: 'b2'),
        _code(NativeCoreException.open),
      );
      await blobs.write('real/item/photo/a1', original);
      await expectLater(
        v.get(kind: 'photo', id: 'a1'),
        _code(NativeCoreException.open),
      );
    });

    test('a tampered blob does not open', () async {
      final v = vault();
      await v.create('2468');
      await v.put(kind: 'note', id: 'a1', plain: note);
      final stored = (await blobs.read('real/item/note/a1'))!;
      stored[stored.length - 1] ^= 1;
      await blobs.write('real/item/note/a1', stored);
      await expectLater(
        v.get(kind: 'note', id: 'a1'),
        _code(NativeCoreException.open),
      );
    });

    test('real and decoy namespaces are independent', () async {
      final real = vault();
      final decoy = vault(ns: 'decoy');
      await real.create('2468');
      await decoy.create('1357');
      await real.put(kind: 'note', id: 'a1', plain: note);
      expect(await decoy.list('note'), isEmpty);
      await expectLater(
        vault(ns: 'decoy').unlock('2468'),
        _code(NativeCoreException.decrypt),
      );
      await expectLater(
        vault().unlock('1357'),
        _code(NativeCoreException.decrypt),
      );
    });

    test('missing items, deletes and replacement', () async {
      final v = vault();
      await v.create('2468');
      await expectLater(
        v.get(kind: 'note', id: 'nope'),
        throwsA(isA<VaultItemNotFoundException>()),
      );
      await v.put(kind: 'note', id: 'a1', plain: Uint8List.fromList([1]));
      await v.put(kind: 'note', id: 'a1', plain: Uint8List.fromList([2]));
      expect(await v.get(kind: 'note', id: 'a1'), [2]);
      await v.put(kind: 'note', id: 'b1', plain: Uint8List(0));
      expect(await v.list('note'), ['a1', 'b1']);
      expect(await v.list('photo'), isEmpty);
      await v.delete(kind: 'note', id: 'a1');
      expect(await v.list('note'), ['b1']);
    });

    test('hostile names are rejected before touching storage', () async {
      final v = vault();
      await v.create('2468');
      for (final id in ['', '../x', 'a/b', 'a|b', 'x' * 65]) {
        await expectLater(
          v.put(kind: 'note', id: id, plain: note),
          throwsA(isA<VaultNameException>()),
        );
      }
      for (final kind in ['', 'Note', 'a/b', 'n-1', 'k' * 33]) {
        await expectLater(
          v.get(kind: kind, id: 'a1'),
          throwsA(isA<VaultNameException>()),
        );
      }
      expect(() => vault(ns: '../x'), throwsA(isA<VaultNameException>()));
    });

    test('a bad-length pepper is an error, not a wrong PIN', () async {
      final short = _ShortPepper();
      await expectLater(
        vault(source: short).create('2468'),
        _code(NativeCoreException.pepper),
      );
      expect(await vault().exists(), isFalse);
    });
  });
}

final class _ShortPepper implements PepperSource {
  @override
  Future<Uint8List> pepper() async => Uint8List(5);
}
