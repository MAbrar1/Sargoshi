/// Vault lock state machine (spec section 3). Pure Dart: no Flutter, no
/// storage, no crypto. It only decides state transitions from a PIN check
/// result. Verifying a PIN, wiping keys and persisting the failure counter
/// are separate jobs that need the Keystore and are NOT implemented here.
library;

sealed class VaultLock {
  const VaultLock();
}

/// [failedAttempts] counts wrong PINs since the last successful open.
/// In this build it lives in memory only. The real counter must live inside
/// Keystore-protected data so it survives a restart.
final class VaultLocked extends VaultLock {
  const VaultLocked({this.failedAttempts = 0});

  final int failedAttempts;
}

final class VaultOpen extends VaultLock {
  const VaultOpen();
}

/// The decoy PIN opened an empty-looking store.
final class VaultDecoyOpen extends VaultLock {
  const VaultDecoyOpen();
}

enum PinCheck { tooShort, accepted, decoy, rejected }

enum VaultAttemptOutcome {
  opened,
  openedDecoy,
  tooShort,
  rejected,

  /// The vault was already open, so nothing changed.
  ignored,
}

const int kMinPinLength = 4;

/// Applies one PIN submission. A too-short PIN is not counted as an attempt.
({VaultLock state, VaultAttemptOutcome outcome}) submitPin(
  VaultLock current,
  PinCheck check,
) {
  switch (current) {
    case VaultOpen() || VaultDecoyOpen():
      return (state: current, outcome: VaultAttemptOutcome.ignored);
    case VaultLocked(:final failedAttempts):
      return switch (check) {
        PinCheck.tooShort => (
          state: current,
          outcome: VaultAttemptOutcome.tooShort,
        ),
        PinCheck.accepted => (
          state: const VaultOpen(),
          outcome: VaultAttemptOutcome.opened,
        ),
        PinCheck.decoy => (
          state: const VaultDecoyOpen(),
          outcome: VaultAttemptOutcome.openedDecoy,
        ),
        PinCheck.rejected => (
          state: VaultLocked(failedAttempts: failedAttempts + 1),
          outcome: VaultAttemptOutcome.rejected,
        ),
      };
  }
}

/// Locking an open vault clears the counter. Locking a locked vault must NOT
/// reset it, otherwise locking would be a way to dodge the wrong-PIN limit.
VaultLock lockVault(VaultLock current) => switch (current) {
  VaultLocked() => current,
  VaultOpen() || VaultDecoyOpen() => const VaultLocked(),
};

/// Policy only: true once wrong PINs reach [threshold]. Nothing in this build
/// wipes anything. A real wipe is a cryptographic erase (spec section 4).
bool wipeDue(VaultLock current, int threshold) =>
    current is VaultLocked && current.failedAttempts >= threshold;

abstract interface class PinVerifier {
  PinCheck check(String pin);
}
