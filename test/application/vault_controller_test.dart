import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/vault/vault_controller.dart';
import 'package:sargoshi/domain/vault/vault_lock.dart';

void main() {
  late ProviderContainer c;

  setUp(() {
    c = ProviderContainer();
    addTearDown(c.dispose);
  });

  VaultController ctl() => c.read(vaultControllerProvider.notifier);
  VaultLock state() => c.read(vaultControllerProvider);

  test('starts locked with no failed attempts', () {
    expect((state() as VaultLocked).failedAttempts, 0);
  });

  test('demo verifier: short, wrong, right and decoy PINs', () {
    const v = DemoPinVerifier();
    expect(v.check('12'), PinCheck.tooShort);
    expect(v.check('9999'), PinCheck.rejected);
    expect(v.check(DemoPinVerifier.demoPin), PinCheck.accepted);
    expect(v.check(DemoPinVerifier.demoDecoyPin), PinCheck.decoy);
  });

  test('wrong PINs count up, the right PIN opens, lock closes', () {
    expect(ctl().submit('9999'), VaultAttemptOutcome.rejected);
    expect(ctl().submit('8888'), VaultAttemptOutcome.rejected);
    expect((state() as VaultLocked).failedAttempts, 2);
    expect(ctl().submit('1234'), VaultAttemptOutcome.opened);
    expect(state(), isA<VaultOpen>());
    ctl().lock();
    expect((state() as VaultLocked).failedAttempts, 0);
  });

  test('decoy PIN opens the decoy vault', () {
    expect(ctl().submit('0000'), VaultAttemptOutcome.openedDecoy);
    expect(state(), isA<VaultDecoyOpen>());
  });

  test('locking while locked keeps the failure counter', () {
    ctl().submit('9999');
    ctl().lock();
    expect((state() as VaultLocked).failedAttempts, 1);
  });
}
