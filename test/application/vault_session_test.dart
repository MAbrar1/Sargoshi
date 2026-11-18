import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/vault/vault_controller.dart';
import 'package:sargoshi/application/vault/vault_session.dart';
import 'package:sargoshi/domain/vault/vault_lock.dart';

import '../helpers/fake_vault_store.dart';

final class _Verifier implements PinVerifier {
  @override
  Future<PinCheck> check(String pin) async => switch (pin) {
    '2468' => PinCheck.accepted,
    '1357' => PinCheck.decoy,
    _ => PinCheck.rejected,
  };
}

void main() {
  late FakeOpener opener;
  late ProviderContainer c;

  ProviderContainer build(FakeOpener o, {bool teardown = true}) {
    final container = ProviderContainer(
      overrides: [
        pinVerifierProvider.overrideWithValue(_Verifier()),
        vaultContentOpenerProvider.overrideWithValue(o),
      ],
    );
    if (teardown) addTearDown(container.dispose);
    return container;
  }

  VaultController ctl() => c.read(vaultControllerProvider.notifier);

  setUp(() {
    opener = FakeOpener();
    c = build(opener);
  });

  test('the real PIN opens the real content store', () async {
    expect(await ctl().submit('2468'), VaultAttemptOutcome.opened);
    expect(opener.calls, [(pin: '2468', decoy: false)]);
    expect(c.read(vaultContentSessionProvider), same(opener.stores.single));
  });

  test('the decoy PIN opens the decoy content store', () async {
    expect(await ctl().submit('1357'), VaultAttemptOutcome.openedDecoy);
    expect(opener.calls, [(pin: '1357', decoy: true)]);
  });

  test('a wrong PIN never opens any content', () async {
    expect(await ctl().submit('9999'), VaultAttemptOutcome.rejected);
    expect(opener.calls, isEmpty);
    expect(c.read(vaultContentSessionProvider), isNull);
  });

  test('locking clears the session and overwrites the key', () async {
    await ctl().submit('2468');
    final store = opener.stores.single;
    ctl().lock();
    expect(c.read(vaultContentSessionProvider), isNull);
    expect(store.locked, isTrue);
    expect(c.read(vaultControllerProvider), isA<VaultLocked>());
  });

  test('if the content cannot be opened the vault stays locked', () async {
    final failing = FakeOpener(fail: true);
    c = build(failing);
    expect(await ctl().submit('2468'), VaultAttemptOutcome.unavailable);
    expect(c.read(vaultControllerProvider), isA<VaultLocked>());
    expect((c.read(vaultControllerProvider) as VaultLocked).failedAttempts, 0);
    expect(c.read(vaultContentSessionProvider), isNull);
  });

  test('disposing the container locks an open store', () async {
    final local = FakeOpener();
    final container = build(local, teardown: false);
    await container.read(vaultControllerProvider.notifier).submit('2468');
    final store = local.stores.single;
    expect(store.locked, isFalse);
    container.dispose();
    expect(store.locked, isTrue);
  });
}
