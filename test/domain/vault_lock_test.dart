import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/domain/vault/vault_lock.dart';

void main() {
  group('submitPin from locked', () {
    test('accepted opens the vault', () {
      final r = submitPin(const VaultLocked(), PinCheck.accepted);
      expect(r.state, isA<VaultOpen>());
      expect(r.outcome, VaultAttemptOutcome.opened);
    });

    test('decoy opens the decoy vault', () {
      final r = submitPin(const VaultLocked(), PinCheck.decoy);
      expect(r.state, isA<VaultDecoyOpen>());
      expect(r.outcome, VaultAttemptOutcome.openedDecoy);
    });

    test('rejected counts a failed attempt', () {
      var s = const VaultLocked() as VaultLock;
      for (var i = 1; i <= 3; i++) {
        final r = submitPin(s, PinCheck.rejected);
        s = r.state;
        expect((s as VaultLocked).failedAttempts, i);
        expect(r.outcome, VaultAttemptOutcome.rejected);
      }
    });

    test('too short is not counted as an attempt', () {
      final r = submitPin(
        const VaultLocked(failedAttempts: 2),
        PinCheck.tooShort,
      );
      expect((r.state as VaultLocked).failedAttempts, 2);
      expect(r.outcome, VaultAttemptOutcome.tooShort);
    });

    test('a correct PIN after failures opens the vault', () {
      final r = submitPin(
        const VaultLocked(failedAttempts: 5),
        PinCheck.accepted,
      );
      expect(r.state, isA<VaultOpen>());
    });
  });

  group('submitPin when already open', () {
    for (final check in PinCheck.values) {
      test('$check changes nothing from open', () {
        final r = submitPin(const VaultOpen(), check);
        expect(r.state, isA<VaultOpen>());
        expect(r.outcome, VaultAttemptOutcome.ignored);
      });
      test('$check changes nothing from decoy', () {
        final r = submitPin(const VaultDecoyOpen(), check);
        expect(r.state, isA<VaultDecoyOpen>());
        expect(r.outcome, VaultAttemptOutcome.ignored);
      });
    }
  });

  group('lockVault', () {
    test('locks an open vault with a clean counter', () {
      final s = lockVault(const VaultOpen());
      expect(s, isA<VaultLocked>());
      expect((s as VaultLocked).failedAttempts, 0);
    });

    test('locks a decoy vault', () {
      expect(lockVault(const VaultDecoyOpen()), isA<VaultLocked>());
    });

    test('locking a locked vault does NOT reset the failure counter', () {
      final s = lockVault(const VaultLocked(failedAttempts: 7));
      expect((s as VaultLocked).failedAttempts, 7);
    });
  });

  group('wipeDue (policy only)', () {
    test('due at and above the threshold, not below', () {
      expect(wipeDue(const VaultLocked(failedAttempts: 9), 10), isFalse);
      expect(wipeDue(const VaultLocked(failedAttempts: 10), 10), isTrue);
      expect(wipeDue(const VaultLocked(failedAttempts: 11), 10), isTrue);
    });

    test('never due while open', () {
      expect(wipeDue(const VaultOpen(), 1), isFalse);
      expect(wipeDue(const VaultDecoyOpen(), 1), isFalse);
    });
  });
}
