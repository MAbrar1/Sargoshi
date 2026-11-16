import 'package:flutter/services.dart' show FilteringTextInputFormatter;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../application/vault/vault_controller.dart';
import '../../domain/vault/vault_lock.dart';
import '../../l10n/app_localizations.dart';

/// Demo vault screen. The PIN check is a fixed stand-in and nothing is
/// stored or protected; the screen says so. Back locks the vault, then exits.
final class VaultScreen extends ConsumerStatefulWidget {
  const VaultScreen({super.key});

  @override
  ConsumerState<VaultScreen> createState() => _VaultScreenState();
}

class _VaultScreenState extends ConsumerState<VaultScreen> {
  final _pin = TextEditingController();
  VaultAttemptOutcome? _last;

  @override
  void dispose() {
    _pin.dispose();
    super.dispose();
  }

  void _submit() {
    final outcome = ref
        .read(vaultControllerProvider.notifier)
        .submit(_pin.text);
    _pin.clear();
    setState(() => _last = outcome);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lock = ref.watch(vaultControllerProvider);
    final title = switch (lock) {
      VaultLocked() => l10n.vaultLocked,
      VaultOpen() => l10n.vaultOpenDemo,
      VaultDecoyOpen() => l10n.vaultDecoyDemo,
    };
    final message = switch ((_last, lock)) {
      (VaultAttemptOutcome.tooShort, _) => l10n.vaultPinTooShort,
      (VaultAttemptOutcome.rejected, VaultLocked(:final failedAttempts)) =>
        l10n.vaultPinIncorrect(failedAttempts),
      _ => null,
    };

    return PopScope(
      onPopInvokedWithResult: (didPop, _) {
        if (didPop) ref.read(vaultControllerProvider.notifier).lock();
      },
      child: Scaffold(
        appBar: AppBar(title: Text(l10n.vaultTitle)),
        body: ListView(
          padding: const EdgeInsetsDirectional.all(20),
          children: [
            Text(
              l10n.vaultDemoDisclosure,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: 20),
            Text(title, style: Theme.of(context).textTheme.titleLarge),
            const SizedBox(height: 16),
            if (lock is VaultLocked) ...[
              TextField(
                controller: _pin,
                obscureText: true,
                keyboardType: TextInputType.number,
                inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                autocorrect: false,
                enableSuggestions: false,
                enableIMEPersonalizedLearning: false,
                onSubmitted: (_) => _submit(),
                decoration: InputDecoration(hintText: l10n.vaultPinHint),
              ),
              if (message != null) ...[
                const SizedBox(height: 8),
                Text(
                  message,
                  style: TextStyle(color: Theme.of(context).colorScheme.error),
                ),
              ],
              const SizedBox(height: 16),
              FilledButton(onPressed: _submit, child: Text(l10n.vaultUnlock)),
            ] else
              FilledButton.tonal(
                onPressed: () =>
                    ref.read(vaultControllerProvider.notifier).lock(),
                child: Text(l10n.vaultLockAction),
              ),
          ],
        ),
      ),
    );
  }
}
