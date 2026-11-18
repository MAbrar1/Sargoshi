import 'dart:io' show Platform;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/vault/vault_lock.dart';
import '../../platform/vault_pepper_source.dart';
import '../../platform/vault_record_store.dart';
import 'vault_setup.dart';

/// What the vault screen needs from a real PIN store.
abstract interface class VaultBackend {
  PinVerifier get verifier;
  Future<bool> isSetUp();

  /// Throws [VaultSetupException] for invalid input.
  Future<void> setPins({required String pin, String? decoyPin});
}

/// Android backend: Keystore secret, record files and the Rust core.
final class NativeVaultBackend implements VaultBackend {
  NativeVaultBackend._(this._setup, this.verifier);

  factory NativeVaultBackend.android() {
    const library = 'libsargoshi_ffi.so';
    final pepper = KeystorePepperSource();
    final records = ChannelVaultRecordStore();
    return NativeVaultBackend._(
      VaultSetup(libraryPath: library, provisioner: pepper, records: records),
      StoredPinVerifier(
        libraryPath: library,
        pepperSource: pepper,
        records: records,
      ),
    );
  }

  final VaultSetup _setup;

  @override
  final PinVerifier verifier;

  @override
  Future<bool> isSetUp() => _setup.isSetUp();

  @override
  Future<void> setPins({required String pin, String? decoyPin}) =>
      _setup.setPins(pin: pin, decoyPin: decoyPin);
}

/// Null off Android (desktop, tests), where the demo verifier is used and the
/// screen says so.
final vaultBackendProvider = Provider<VaultBackend?>(
  (ref) => Platform.isAndroid ? NativeVaultBackend.android() : null,
);

/// True when a real backend exists and no PIN has been set yet.
final vaultNeedsSetupProvider = FutureProvider.autoDispose<bool>((ref) async {
  final backend = ref.watch(vaultBackendProvider);
  if (backend == null) return false;
  return !await backend.isSetUp();
});
