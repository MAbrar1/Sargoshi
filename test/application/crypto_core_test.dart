import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/crypto/crypto_provider.dart';
import 'package:sargoshi/core/config/app_config.dart';
import 'package:sargoshi/domain/crypto/crypto_core.dart';
import 'package:sargoshi/testing/fake_crypto_core.dart';

void main() {
  test('fake core round-trips and is not production grade', () async {
    const core = FakeCryptoCore();
    expect(core.isProductionGrade, isFalse);
    final plain = Uint8List.fromList([1, 2, 3, 4]);
    final sealed = await core.seal(contactId: 'maya', plaintext: plain);
    expect(await core.open(contactId: 'maya', message: sealed), plain);
  });

  test('fake core rejects foreign or truncated input', () async {
    const core = FakeCryptoCore();
    for (final bytes in [
      Uint8List(0),
      Uint8List.fromList([1, 2, 3, 4, 5]),
    ]) {
      expect(
        () => core.open(contactId: 'x', message: SealedMessage(bytes)),
        throwsA(isA<CryptoOpenException>()),
      );
    }
  });

  test('provider serves the fake core by default and the app says so', () {
    final c = ProviderContainer();
    addTearDown(c.dispose);
    expect(c.read(cryptoCoreProvider).isProductionGrade, isFalse);
    expect(AppConfig.isPlaceholderDomain, isTrue);
  });
}
