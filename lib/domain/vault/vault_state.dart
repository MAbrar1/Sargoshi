import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../l10n/app_localizations.dart';

/// Stage 2 vault shell: honest, local-only, not a real secure store.
/// The implementation is intentionally minimal and remains behind a visible
/// demo banner until a real encrypted vault is connected.

enum VaultLockState { locked, unlocked, decoy, error }

final class VaultSession {
  const VaultSession({
    required this.pinLength,
    required this.state,
    required this.lastError,
    required this.isHardwareBacked,
  });

  final int pinLength;
  final VaultLockState state;
  final String? lastError;
  final bool isHardwareBacked;

  VaultSession copyWith({
    int? pinLength,
    VaultLockState? state,
    String? lastError,
    bool? isHardwareBacked,
  }) => VaultSession(
    pinLength: pinLength ?? this.pinLength,
    state: state ?? this.state,
    lastError: lastError ?? this.lastError,
    isHardwareBacked: isHardwareBacked ?? this.isHardwareBacked,
  );
}

final vaultSessionProvider = StateNotifierProvider<VaultSessionController, VaultSession>(
  (ref) => VaultSessionController(),
);

final class VaultSessionController extends StateNotifier<VaultSession> {
  VaultSessionController()
      : super(
          const VaultSession(
            pinLength: 4,
            state: VaultLockState.locked,
            lastError: null,
            isHardwareBacked: false,
          ),
        );

  void unlock(String pin, AppLocalizations l10n) {
    final normalized = pin.trim();
    if (normalized.length < 4) {
      state = state.copyWith(
        state: VaultLockState.error,
        lastError: l10n.vaultPinTooShort,
      );
      return;
    }
    state = state.copyWith(
      state: VaultLockState.unlocked,
      lastError: null,
    );
  }

  void decoy() {
    state = state.copyWith(state: VaultLockState.decoy, lastError: null);
  }

  void lock() {
    state = state.copyWith(
      state: VaultLockState.locked,
      lastError: null,
    );
  }

  void wipe() {
    state = state.copyWith(
      state: VaultLockState.locked,
      lastError: null,
    );
  }
}
