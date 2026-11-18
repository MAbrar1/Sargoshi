import 'dart:io' show Platform;
import 'dart:isolate';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../platform/native/sargoshi_native.dart';
import '../../platform/vault_blob_store.dart';
import '../../platform/vault_pepper_source.dart';
import 'native_pin_verifier.dart' show PepperSource;
import 'vault_content.dart';

/// Opens the encrypted content store after the PIN verifier accepted [pin].
/// A failure means the content could not be opened, never "wrong PIN".
abstract interface class VaultContentOpener {
  Future<VaultItemStore> open({required String pin, required bool decoy});
}

/// Android opener: Keystore secret, blob files and the Rust core. The slow key
/// derivation runs on a background isolate.
final class NativeVaultContentOpener implements VaultContentOpener {
  NativeVaultContentOpener({
    required this.libraryPath,
    required this.pepperSource,
    required this.blobs,
  });

  factory NativeVaultContentOpener.android() => NativeVaultContentOpener(
    libraryPath: 'libsargoshi_ffi.so',
    pepperSource: KeystorePepperSource(),
    blobs: ChannelVaultBlobStore(),
  );

  final String libraryPath;
  final PepperSource pepperSource;
  final ChannelVaultBlobStore blobs;

  @override
  Future<VaultItemStore> open({
    required String pin,
    required bool decoy,
  }) async {
    final path = libraryPath;
    final content = VaultContent(
      native: SargoshiNative.open(path),
      pepperSource: pepperSource,
      blobs: blobs,
      namespace: decoy ? 'decoy' : 'real',
      wrapKey: (master, wrapPin, pepper) => Isolate.run(
        () => SargoshiNative.open(path).wrapKey(master, wrapPin, pepper),
      ),
      unwrapKey: (blob, unwrapPin, pepper) => Isolate.run(
        () => SargoshiNative.open(path).unwrapKey(blob, unwrapPin, pepper),
      ),
    );
    await content.unlockOrCreate(pin);
    return content;
  }
}

/// Null off Android (desktop, tests): the vault then stores nothing.
final vaultContentOpenerProvider = Provider<VaultContentOpener?>(
  (ref) => Platform.isAndroid ? NativeVaultContentOpener.android() : null,
);

/// The unlocked content store, or null while the vault is locked. Replacing or
/// clearing it overwrites the previous key.
final class VaultContentSession extends Notifier<VaultItemStore?> {
  VaultItemStore? _current;

  @override
  VaultItemStore? build() {
    ref.onDispose(() => _current?.lock());
    return null;
  }

  void open(VaultItemStore store) {
    if (!identical(_current, store)) _current?.lock();
    _current = store;
    state = store;
  }

  void close() {
    _current?.lock();
    _current = null;
    state = null;
  }
}

final vaultContentSessionProvider =
    NotifierProvider<VaultContentSession, VaultItemStore?>(
      VaultContentSession.new,
    );
