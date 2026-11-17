import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/platform/native/sargoshi_native.dart';

/// Runs against the host build of `sargoshi_ffi`. CI sets SARGOSHI_FFI_LIB.
/// Without it the group is skipped and says why.
void main() {
  final path = Platform.environment['SARGOSHI_FFI_LIB'];
  final skip = path == null
      ? 'SARGOSHI_FFI_LIB not set; build rust/sargoshi_ffi to run'
      : false;

  group('native core over dart:ffi', skip: skip, () {
    late SargoshiNative native;
    setUp(() => native = SargoshiNative.open(path!));

    test('pad and unpad round-trip', () {
      final plain = Uint8List.fromList(List.generate(300, (i) => i % 251));
      final padded = native.pad(plain);
      expect(padded.length, 512);
      expect(native.unpad(padded), plain);
    });

    test('empty input works', () {
      expect(native.unpad(native.pad(const [])), isEmpty);
    });

    test('malformed padding reports a format code', () {
      expect(
        () => native.unpad(const [1, 2]),
        throwsA(
          isA<NativeCoreException>().having(
            (e) => e.code,
            'code',
            NativeCoreException.format,
          ),
        ),
      );
    });

    test('recovery round-trip, wrong passphrase and weak passphrase', () {
      final keys = Uint8List.fromList([9, 8, 7, 6]);
      final file = native.createRecovery(keys, 'correct horse battery');
      expect(native.restoreRecovery(file, 'correct horse battery'), keys);
      expect(
        () => native.restoreRecovery(file, 'wrong horse battery'),
        throwsA(
          isA<NativeCoreException>().having(
            (e) => e.code,
            'code',
            NativeCoreException.decrypt,
          ),
        ),
      );
      expect(
        () => native.createRecovery(keys, 'short'),
        throwsA(
          isA<NativeCoreException>().having(
            (e) => e.code,
            'code',
            NativeCoreException.weakPassphrase,
          ),
        ),
      );
    });

    test('non-ASCII passphrase survives UTF-8 encoding', () {
      final keys = Uint8List.fromList([1, 2, 3]);
      const pass = 'پاسورڈ-محفوظ-۱۲۳۴';
      final file = native.createRecovery(keys, pass);
      expect(native.restoreRecovery(file, pass), keys);
    });
  });
}
