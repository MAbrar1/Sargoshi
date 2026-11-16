import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/vault/vault_state.dart';
import '../../l10n/app_localizations.dart';

/// Stage 2 vault sensor stub: the real keystore and SQLCipher handoff are not yet
/// wired. This class documents the boundary and keeps the UI honest.
final class KeystoreFacade {
  const KeystoreFacade();

  Future<bool> get isHardwareBacked async => false;

  Future<void> wipe() async {}

  Future<String?> readVaultKey() async => null;

  Future<void> writeVaultKey(String key) async {}
}

final keystoreFacadeProvider = Provider<KeystoreFacade>(
  (ref) => const KeystoreFacade(),
);

final vaultPinValidationProvider = Provider.family<bool, String>((ref, pin) {
  final trimmed = pin.trim();
  return trimmed.length >= 4;
});

final class VaultScreen extends ConsumerWidget {
  const VaultScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final vaultState = ref.watch(vaultSessionProvider);
    final pinController = TextEditingController();

    return Scaffold(
      appBar: AppBar(title: Text(l10n.vaultTitle)),
      body: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              switch (vaultState.state) {
                VaultLockState.locked => l10n.vaultLocked,
                VaultLockState.unlocked => l10n.vaultUnlocked,
                VaultLockState.decoy => l10n.vaultDecoy,
                VaultLockState.error => l10n.vaultError,
              },
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 20),
            TextField(
              controller: pinController,
              keyboardType: TextInputType.number,
              obscureText: true,
              decoration: InputDecoration(
                hintText: l10n.vaultPinHint,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(16),
                ),
              ),
              maxLength: vaultState.pinLength,
            ),
            if (vaultState.lastError != null) ...[
              const SizedBox(height: 12),
              Text(vaultState.lastError!),
            ],
            const SizedBox(height: 20),
            Row(
              children: [
                Expanded(
                  child: ElevatedButton(
                    onPressed: () => ref
                        .read(vaultSessionProvider.notifier)
                        .unlock(pinController.text, l10n),
                    child: Text(l10n.vaultUnlock),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: OutlinedButton(
                    onPressed: () =>
                        ref.read(vaultSessionProvider.notifier).decoy(),
                    child: Text(l10n.vaultDecoyAction),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
