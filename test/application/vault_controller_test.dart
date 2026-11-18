import 'dart:async';

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

  test('demo verifier: short, wrong, right and decoy PINs', () async {
    const v = DemoPinVerifier();
    expect(await v.check('12'), PinCheck.tooShort);
    expect(await v.check('9999'), PinCheck.rejected);
    expect(await v.check(DemoPinVerifier.demoPin), PinCheck.accepted);
    expect(await v.check(DemoPinVerifier.demoDecoyPin), PinCheck.decoy);
  });

  test('wrong PINs count up, the right PIN opens, lock closes', () async {
    expect(await ctl().submit('9999'), VaultAttemptOutcome.rejected);
    expect(await ctl().submit('8888'), VaultAttemptOutcome.rejected);
    expect((state() as VaultLocked).failedAttempts, 2);
    expect(await ctl().submit('1234'), VaultAttemptOutcome.opened);
    expect(state(), isA<VaultOpen>());
    ctl().lock();
    expect((state() as VaultLocked).failedAttempts, 0);
  });

  test('decoy PIN opens the decoy vault', () async {
    expect(await ctl().submit('0000'), VaultAttemptOutcome.openedDecoy);
    expect(state(), isA<VaultDecoyOpen>());
  });

  test('locking while locked keeps the failure counter', () async {
    await ctl().submit('9999');
    ctl().lock();
    expect((state() as VaultLocked).failedAttempts, 1);
  });

  group('with a controllable verifier', () {
    late Completer<PinCheck> gate;
    Object? failure;

    setUp(() {
      gate = Completer<PinCheck>();
      failure = null;
      c.dispose();
      c = ProviderContainer(
        overrides: [
          pinVerifierProvider.overrideWithValue(
            _Gated(() {
              if (failure != null) throw failure!;
              return gate.future;
            }),
          ),
        ],
      );
    });

    test('a failing check is unavailable, not a wrong PIN', () async {
      failure = StateError('record unusable');
      expect(await ctl().submit('1234'), VaultAttemptOutcome.unavailable);
      expect((state() as VaultLocked).failedAttempts, 0);
    });

    test('overlapping attempts are ignored', () async {
      final first = ctl().submit('1234');
      expect(await ctl().submit('1234'), VaultAttemptOutcome.ignored);
      gate.complete(PinCheck.accepted);
      expect(await first, VaultAttemptOutcome.opened);
    });

    test('locking while a check runs discards its result', () async {
      final pending = ctl().submit('1234');
      ctl().lock();
      gate.complete(PinCheck.accepted);
      expect(await pending, VaultAttemptOutcome.ignored);
      expect(state(), isA<VaultLocked>());
    });
  });
}

final class _Gated implements PinVerifier {
  _Gated(this._run);
  final Future<PinCheck> Function() _run;

  @override
  Future<PinCheck> check(String pin) => _run();
}
