import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/config/app_config.dart';
import '../../domain/crypto/crypto_core.dart';
import '../../testing/fake_crypto_core.dart';

/// The only production-tree file allowed to import the fake core.
final cryptoCoreProvider = Provider<CryptoCore>((ref) {
  if (AppConfig.useFakeCrypto) return const FakeCryptoCore();
  throw StateError('No production CryptoCore is registered yet.');
});
