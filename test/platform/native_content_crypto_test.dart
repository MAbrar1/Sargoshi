import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/platform/native/sargoshi_native.dart';

Matcher _code(int code) => throwsA(
  isA<NativeCoreException>().having((e) => e.code, 'code', code),
);

/// Runs against the host build of `sargoshi_ffi`. CI sets SARGOSHI_FFI_LIB.
void main() {
  final path = Platform.environment['SARGOSHI_FFI_LIB'];
  final skip = path == null
      ? 'SARGOSHI_FFI_LIB not set; build rust/sargoshi_ffi to run'
      : false;

  group('native content crypto', skip: skip, () {
    late SargoshiNative native;
    final pepper = Uint8List.fromList(List.filled(32, 5));
    setUp(() => native = SargoshiNative.open(path!));

    test('generated keys are 32 bytes and differ', () {
      final a = native.generateKey();
      final b = native.generateKey();
      expect(a.length, 32);
      expect(a, isNot(b));
    });

    test('seal and open round-trip, including empty and non-ASCII', () {
      final key = native.generateKey();
      for (final plain in [
        Uint8List(0),
        Uint8List.fromList('نرمی سے کہو'.codeUnits),
        Uint8List.fromList(List.generate(5000, (i) => i % 256)),
      ]) {
        final sealed = native.seal(key, [1, 2], plain);
        expect(native.open(key, [1, 2], sealed), plain);
      }
    });

    test('wrong key, wrong context and tampering fail the same way', () {
      final key = native.generateKey();
      final sealed = native.seal(key, [1], [9, 9, 9]);
      expect(
        () => native.open(native.generateKey(), [1], sealed),
        _code(NativeCoreException.open),
      );
      expect(() => native.open(key, [2], sealed), _code(NativeCoreException.open));
      final tampered = Uint8List.fromList(sealed)..[sealed.length - 1] ^= 1;
      expect(() => native.open(key, [1], tampered), _code(NativeCoreException.open));
      expect(() => native.seal([1, 2], [], [1]), _code(NativeCoreException.key));
    });

    test('wrapped master key unwraps only with the right PIN and pepper', () {
      final master = native.generateKey();
      final wrapped = native.wrapKey(master, '2468', pepper);
      expect(native.unwrapKey(wrapped, '2468', pepper), master);
      expect(
        () => native.unwrapKey(wrapped, '2469', pepper),
        _code(NativeCoreException.decrypt),
      );
      expect(
        () => native.unwrapKey(wrapped, '2468', Uint8List(32)),
        _code(NativeCoreException.decrypt),
      );
      expect(
        () => native.wrapKey(master, '12', pepper),
        _code(NativeCoreException.pinTooShort),
      );
    });

    test('changing the PIN keeps items readable', () {
      final master = native.generateKey();
      final item = native.seal(master, [7], [4, 5, 6]);
      final first = native.wrapKey(master, '2468', pepper);
      final recovered = native.unwrapKey(first, '2468', pepper);
      final second = native.wrapKey(recovered, '8642', pepper);
      final again = native.unwrapKey(second, '8642', pepper);
      expect(native.open(again, [7], item), [4, 5, 6]);
    });
  });
}
