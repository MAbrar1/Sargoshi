import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/vault/native_pin_verifier.dart';
import 'package:sargoshi/domain/vault/vault_lock.dart';
import 'package:sargoshi/platform/native/sargoshi_native.dart';

final class _Pepper implements PepperSource {
  _Pepper(this.bytes);
  final Uint8List bytes;
  @override
  Future<Uint8List> pepper() async => bytes;
}

/// Runs against the host build of `sargoshi_ffi`. CI sets SARGOSHI_FFI_LIB.
void main() {
  final path = Platform.environment['SARGOSHI_FFI_LIB'];
  final skip = path == null
      ? 'SARGOSHI_FFI_LIB not set; build rust/sargoshi_ffi to run'
      : false;

  group('native PIN verifier', skip: skip, () {
    late SargoshiNative native;
    final pepper = _Pepper(Uint8List.fromList(List.filled(32, 5)));
    setUp(() => native = SargoshiNative.open(path!));

    Future<NativePinVerifier> build({bool decoy = true}) async {
      final real = await NativePinVerifier.enroll(
        native: native,
        pepperSource: pepper,
        pin: '2468',
      );
      final fake = await NativePinVerifier.enroll(
        native: native,
        pepperSource: pepper,
        pin: '1357',
      );
      return NativePinVerifier(
        native: native,
        pepperSource: pepper,
        realRecord: real,
        decoyRecord: decoy ? fake : null,
      );
    }

    test('real PIN, decoy PIN, wrong PIN and short PIN', () async {
      final v = await build();
      expect(await v.check('2468'), PinCheck.accepted);
      expect(await v.check('1357'), PinCheck.decoy);
      expect(await v.check('9999'), PinCheck.rejected);
      expect(await v.check('12'), PinCheck.tooShort);
    });

    test('without a decoy record the decoy PIN is just wrong', () async {
      final v = await build(decoy: false);
      expect(await v.check('1357'), PinCheck.rejected);
    });

    test('a different pepper rejects the right PIN', () async {
      final v = await build();
      final other = NativePinVerifier(
        native: native,
        pepperSource: _Pepper(Uint8List.fromList(List.filled(32, 6))),
        realRecord: v.realRecord,
      );
      expect(await other.check('2468'), PinCheck.rejected);
    });

    test('a corrupt record is an error, never "wrong PIN"', () async {
      final v = await build();
      final broken = NativePinVerifier(
        native: native,
        pepperSource: pepper,
        realRecord: Uint8List.fromList([1, 2, 3]),
      );
      expect(v.realRecord.length, greaterThan(3));
      await expectLater(
        broken.check('2468'),
        throwsA(isA<NativeCoreException>()),
      );
    });

    test(
      'isolate verifier gives the same answers off the main isolate',
      () async {
        final v = await build();
        final iso = IsolatePinVerifier(
          libraryPath: path!,
          pepperSource: pepper,
          realRecord: v.realRecord,
          decoyRecord: v.decoyRecord,
        );
        expect(await iso.check('2468'), PinCheck.accepted);
        expect(await iso.check('1357'), PinCheck.decoy);
        expect(await iso.check('9999'), PinCheck.rejected);
        expect(await iso.check('12'), PinCheck.tooShort);
      },
    );

    test('a pepper of the wrong length is an error', () async {
      final bad = NativePinVerifier(
        native: native,
        pepperSource: _Pepper(Uint8List(5)),
        realRecord: (await build()).realRecord,
      );
      await expectLater(bad.check('2468'), throwsA(isA<NativeCoreException>()));
    });
  });
}
