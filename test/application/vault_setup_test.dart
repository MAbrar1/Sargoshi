import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/vault/native_pin_verifier.dart';
import 'package:sargoshi/application/vault/vault_setup.dart';
import 'package:sargoshi/domain/vault/vault_lock.dart';
import 'package:sargoshi/domain/vault/vault_records.dart';

final class _MemoryRecords implements VaultRecordStore {
  final map = <VaultSlot, Uint8List>{};

  @override
  Future<Uint8List?> read(VaultSlot slot) async => map[slot];

  @override
  Future<void> write(VaultSlot slot, Uint8List record) async =>
      map[slot] = record;

  @override
  Future<void> delete(VaultSlot slot) async => map.remove(slot);
}

final class _Pepper implements PepperSource, PepperProvisioner {
  _Pepper(this.bytes);
  Uint8List? bytes;

  @override
  Future<Uint8List> pepper() async => bytes ?? (throw StateError('missing'));

  @override
  Future<Uint8List> createIfMissing() async =>
      bytes ??= Uint8List.fromList(List.filled(32, 4));
}

/// Runs against the host build of `sargoshi_ffi`. CI sets SARGOSHI_FFI_LIB.
void main() {
  final path = Platform.environment['SARGOSHI_FFI_LIB'];
  final skip = path == null
      ? 'SARGOSHI_FFI_LIB not set; build rust/sargoshi_ffi to run'
      : false;

  group('vault setup and stored verifier', skip: skip, () {
    late _MemoryRecords records;
    late _Pepper pepper;
    late VaultSetup setup;
    late StoredPinVerifier verifier;

    setUp(() {
      records = _MemoryRecords();
      pepper = _Pepper(null);
      setup = VaultSetup(
        libraryPath: path!,
        provisioner: pepper,
        records: records,
      );
      verifier = StoredPinVerifier(
        libraryPath: path,
        pepperSource: pepper,
        records: records,
      );
    });

    test('before setup the vault is not set up, not "wrong PIN"', () async {
      expect(await setup.isSetUp(), isFalse);
      await expectLater(
        verifier.check('2468'),
        throwsA(isA<VaultNotSetUpException>()),
      );
    });

    test('set a PIN with a decoy, then check all outcomes', () async {
      await setup.setPins(pin: '2468', decoyPin: '1357');
      expect(await setup.isSetUp(), isTrue);
      expect(await verifier.check('2468'), PinCheck.accepted);
      expect(await verifier.check('1357'), PinCheck.decoy);
      expect(await verifier.check('9999'), PinCheck.rejected);
      expect(await verifier.check('12'), PinCheck.tooShort);
    });

    test('changing the PIN without a decoy removes the old decoy', () async {
      await setup.setPins(pin: '2468', decoyPin: '1357');
      await setup.setPins(pin: '8642');
      expect(records.map.containsKey(VaultSlot.decoy), isFalse);
      expect(await verifier.check('8642'), PinCheck.accepted);
      expect(await verifier.check('2468'), PinCheck.rejected);
      expect(await verifier.check('1357'), PinCheck.rejected);
    });

    test('invalid input writes nothing', () async {
      await expectLater(
        setup.setPins(pin: '12'),
        throwsA(
          isA<VaultSetupException>().having(
            (e) => e.error,
            'error',
            VaultSetupError.tooShort,
          ),
        ),
      );
      await expectLater(
        setup.setPins(pin: '2468', decoyPin: '2468'),
        throwsA(
          isA<VaultSetupException>().having(
            (e) => e.error,
            'error',
            VaultSetupError.decoySameAsReal,
          ),
        ),
      );
      expect(records.map, isEmpty);
      expect(pepper.bytes, isNull);
    });

    test('a lost secret with surviving records is an error', () async {
      await setup.setPins(pin: '2468');
      pepper.bytes = null;
      await expectLater(verifier.check('2468'), throwsStateError);
    });
  });
}
