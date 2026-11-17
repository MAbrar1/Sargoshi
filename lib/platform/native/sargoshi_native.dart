import 'dart:convert';
import 'dart:ffi';
import 'dart:io' show Platform;
import 'dart:typed_data';

/// ABI version this Dart code was written against. Must equal
/// `sg_abi_version()` in `rust/sargoshi_ffi`.
const int kExpectedAbiVersion = 2;

/// Failure reported by the native core. Carries only a code, never a message
/// that could describe secrets.
final class NativeCoreException implements Exception {
  const NativeCoreException(this.code);

  final int code;

  static const arg = 1;
  static const format = 2;
  static const decrypt = 3;
  static const weakPassphrase = 4;
  static const params = 5;
  static const internal = 6;
  static const tooLarge = 7;
  static const pinTooShort = 8;
  static const pepper = 9;

  @override
  String toString() => 'NativeCoreException($code)';
}

final class _SgBuf extends Struct {
  external Pointer<Uint8> ptr;

  @IntPtr()
  external int len;
}

typedef _AllocC = Pointer<Uint8> Function(IntPtr);
typedef _Alloc = Pointer<Uint8> Function(int);
typedef _DeallocC = Void Function(Pointer<Uint8>);
typedef _Dealloc = void Function(Pointer<Uint8>);
typedef _OneInC = Int32 Function(Pointer<Uint8>, IntPtr, Pointer<_SgBuf>);
typedef _OneIn = int Function(Pointer<Uint8>, int, Pointer<_SgBuf>);
typedef _TwoInC =
    Int32 Function(
      Pointer<Uint8>,
      IntPtr,
      Pointer<Uint8>,
      IntPtr,
      Pointer<_SgBuf>,
    );
typedef _TwoIn =
    int Function(Pointer<Uint8>, int, Pointer<Uint8>, int, Pointer<_SgBuf>);
typedef _VerifyC =
    Int32 Function(
      Pointer<Uint8>,
      IntPtr,
      Pointer<Uint8>,
      IntPtr,
      Pointer<Uint8>,
      IntPtr,
    );
typedef _Verify =
    int Function(Pointer<Uint8>, int, Pointer<Uint8>, int, Pointer<Uint8>, int);
typedef _ReleaseC = Void Function(Pointer<_SgBuf>);
typedef _Release = void Function(Pointer<_SgBuf>);
typedef _AbiC = Uint32 Function();
typedef _Abi = int Function();

/// Typed access to `libsargoshi_ffi`. Inputs are copied into native memory
/// that Rust zeroes on release; results are copied out and the native buffer
/// is zeroed and freed before returning.
final class SargoshiNative {
  SargoshiNative._(DynamicLibrary lib)
    : _alloc = lib.lookupFunction<_AllocC, _Alloc>('sg_alloc'),
      _dealloc = lib.lookupFunction<_DeallocC, _Dealloc>('sg_dealloc'),
      _pad = lib.lookupFunction<_OneInC, _OneIn>('sg_pad'),
      _unpad = lib.lookupFunction<_OneInC, _OneIn>('sg_unpad'),
      _create = lib.lookupFunction<_TwoInC, _TwoIn>('sg_recovery_create'),
      _restore = lib.lookupFunction<_TwoInC, _TwoIn>('sg_recovery_restore'),
      _enroll = lib.lookupFunction<_TwoInC, _TwoIn>('sg_vault_enroll'),
      _verify = lib.lookupFunction<_VerifyC, _Verify>('sg_vault_verify'),
      _release = lib.lookupFunction<_ReleaseC, _Release>('sg_buf_release');

  /// Opens the library at [path] and refuses a mismatched ABI.
  factory SargoshiNative.open(String path) {
    final lib = DynamicLibrary.open(path);
    final version = lib.lookupFunction<_AbiC, _Abi>('sg_abi_version')();
    if (version != kExpectedAbiVersion) {
      throw StateError('Native ABI $version, expected $kExpectedAbiVersion');
    }
    return SargoshiNative._(lib);
  }

  /// Opens the library packaged with the app (Android loads it by soname).
  factory SargoshiNative.openDefault() {
    if (!Platform.isAndroid) {
      throw UnsupportedError('Native core is packaged for Android only');
    }
    return SargoshiNative.open('libsargoshi_ffi.so');
  }

  final _Alloc _alloc;
  final _Dealloc _dealloc;
  final _OneIn _pad;
  final _OneIn _unpad;
  final _TwoIn _create;
  final _TwoIn _restore;
  final _TwoIn _enroll;
  final _Verify _verify;
  final _Release _release;

  Pointer<Uint8> _in(List<int> bytes) {
    final p = _alloc(bytes.length);
    if (p == nullptr) {
      throw const NativeCoreException(NativeCoreException.internal);
    }
    p.asTypedList(bytes.length).setAll(0, bytes);
    return p;
  }

  Uint8List _call(
    List<int> a,
    List<int>? b,
    int Function(Pointer<Uint8>, Pointer<Uint8>?, Pointer<_SgBuf>) invoke,
  ) {
    final pa = _in(a);
    final pb = b == null ? null : _in(b);
    final out = _alloc(sizeOf<_SgBuf>()).cast<_SgBuf>();
    try {
      final code = invoke(pa, pb, out);
      if (code != 0) throw NativeCoreException(code);
      final buf = out.ref;
      return Uint8List.fromList(buf.ptr.asTypedList(buf.len));
    } finally {
      _release(out);
      _dealloc(out.cast());
      _dealloc(pa);
      if (pb != null) _dealloc(pb);
    }
  }

  Uint8List pad(List<int> plain) =>
      _call(plain, null, (a, _, o) => _pad(a, plain.length, o));

  Uint8List unpad(List<int> padded) =>
      _call(padded, null, (a, _, o) => _unpad(a, padded.length, o));

  Uint8List createRecovery(List<int> identityKeys, String passphrase) {
    final pass = utf8.encode(passphrase);
    return _call(
      identityKeys,
      pass,
      (a, b, o) => _create(a, identityKeys.length, b!, pass.length, o),
    );
  }

  Uint8List restoreRecovery(List<int> file, String passphrase) {
    final pass = utf8.encode(passphrase);
    return _call(
      file,
      pass,
      (a, b, o) => _restore(a, file.length, b!, pass.length, o),
    );
  }

  /// Creates a vault verifier record for [pin], keyed with a 32-byte [pepper].
  /// Slow by design (Argon2id): call off the UI isolate.
  Uint8List enrollVault(String pin, List<int> pepper) {
    final p = utf8.encode(pin);
    return _call(
      p,
      pepper,
      (a, b, o) => _enroll(a, p.length, b!, pepper.length, o),
    );
  }

  /// True when [pin] matches [record]; false for a wrong PIN. Throws
  /// [NativeCoreException] when the record or pepper is unusable, which is not
  /// the same as a wrong PIN.
  bool verifyVault(List<int> record, String pin, List<int> pepper) {
    final p = utf8.encode(pin);
    final pr = _in(record);
    final pp = _in(p);
    final pk = _in(pepper);
    try {
      final code = _verify(
        pr,
        record.length,
        pp,
        p.length,
        pk,
        pepper.length,
      );
      if (code == 0) return true;
      if (code == _noMatch) return false;
      throw NativeCoreException(code);
    } finally {
      _dealloc(pr);
      _dealloc(pp);
      _dealloc(pk);
    }
  }
}

const int _noMatch = 10;
