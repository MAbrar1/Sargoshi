import 'dart:async';

import 'package:flutter/services.dart' show FilteringTextInputFormatter;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../application/vault/attempt_limiter.dart';
import '../../application/vault/vault_backend.dart';
import '../../application/vault/vault_controller.dart';
import '../../application/vault/vault_session.dart';
import '../../application/vault/vault_setup.dart';
import '../../domain/vault/vault_lock.dart';
import '../../l10n/app_localizations.dart';
import 'vault_change_pin_view.dart';
import 'vault_notes_view.dart';

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
  final _confirm = TextEditingController();
  bool _busy = false;
  Timer? _tick;
  String? _setupError;

  @override
  void dispose() {
    _tick?.cancel();
    _pin.dispose();
    _confirm.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (_busy) return;
    final pin = _pin.text;
    setState(() => _busy = true);
    final outcome = await ref
        .read(vaultControllerProvider.notifier)
        .submit(pin);
    if (!mounted) return;
    _pin.clear();
    setState(() {
      _last = outcome;
      _busy = false;
    });
  }

  Future<void> _setPin(AppLocalizations l10n) async {
    if (_busy) return;
    final pin = _pin.text;
    if (pin != _confirm.text) {
      setState(() => _setupError = l10n.vaultPinMismatch);
      return;
    }
    setState(() {
      _busy = true;
      _setupError = null;
    });
    String? error;
    try {
      await ref.read(vaultBackendProvider)!.setPins(pin: pin);
    } on VaultSetupException {
      error = l10n.vaultPinTooShort;
    } on Object {
      error = l10n.vaultSetupFailed;
    }
    if (!mounted) return;
    _pin.clear();
    _confirm.clear();
    if (error == null) ref.invalidate(vaultNeedsSetupProvider);
    setState(() {
      _busy = false;
      _setupError = error;
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lock = ref.watch(vaultControllerProvider);
    final isReal = ref.watch(vaultBackendProvider) != null;
    final hasContent = ref.watch(vaultContentOpenerProvider) != null;
    final store = ref.watch(vaultContentSessionProvider);
    final needsSetup = ref.watch(vaultNeedsSetupProvider).value ?? false;
    final status = ref.watch(attemptStatusProvider).value;
    final throttled = status?.isThrottled ?? false;
    // Tick once a second while a delay is running so the time left stays true.
    ref.listen<AsyncValue<AttemptStatus>>(attemptStatusProvider, (_, next) {
      final waiting = next.value?.isThrottled ?? false;
      if (waiting && _tick == null) {
        _tick = Timer.periodic(
          const Duration(seconds: 1),
          (_) => ref.invalidate(attemptStatusProvider),
        );
      } else if (!waiting) {
        _tick?.cancel();
        _tick = null;
      }
    });
    final title = switch (lock) {
      VaultLocked() => l10n.vaultLocked,
      VaultOpen() => l10n.vaultOpenDemo,
      VaultDecoyOpen() => l10n.vaultDecoyDemo,
    };
    final message = switch ((_last, lock)) {
      (VaultAttemptOutcome.tooShort, _) => l10n.vaultPinTooShort,
      (VaultAttemptOutcome.unavailable, _) => l10n.vaultCheckUnavailable,
      (VaultAttemptOutcome.throttled, _) => null,
      (VaultAttemptOutcome.rejected, VaultLocked(:final failedAttempts)) =>
        l10n.vaultPinIncorrect(failedAttempts),
      _ => null,
    };

    final shownMessage = throttled
        ? l10n.vaultThrottled(_formatWait(status!.remaining))
        : message;

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
              hasContent
                  ? l10n.vaultContentDisclosure
                  : isReal
                  ? l10n.vaultRealDisclosure
                  : l10n.vaultDemoDisclosure,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: 20),
            if (needsSetup) ...[
              Text(
                l10n.vaultSetupTitle,
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 8),
              Text(l10n.vaultSetupHint),
              const SizedBox(height: 16),
              _PinField(
                controller: _pin,
                hint: l10n.vaultPinHint,
                enabled: !_busy,
              ),
              const SizedBox(height: 12),
              _PinField(
                controller: _confirm,
                hint: l10n.vaultPinConfirmHint,
                enabled: !_busy,
                onSubmitted: (_) => _setPin(l10n),
              ),
              if (_setupError != null) ...[
                const SizedBox(height: 8),
                Text(
                  _setupError!,
                  style: TextStyle(color: Theme.of(context).colorScheme.error),
                ),
              ],
              const SizedBox(height: 16),
              FilledButton(
                onPressed: _busy ? null : () => _setPin(l10n),
                child: Text(_busy ? l10n.vaultChecking : l10n.vaultSetupAction),
              ),
            ] else ...[
              Text(title, style: Theme.of(context).textTheme.titleLarge),
              const SizedBox(height: 16),
              if (lock is VaultLocked) ...[
                _PinField(
                  controller: _pin,
                  hint: l10n.vaultPinHint,
                  enabled: !_busy,
                  onSubmitted: (_) => _submit(),
                ),
                if (shownMessage != null) ...[
                  const SizedBox(height: 8),
                  Text(
                    shownMessage,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
                ],
                const SizedBox(height: 16),
                FilledButton(
                  onPressed: _busy || throttled ? null : _submit,
                  child: Text(_busy ? l10n.vaultChecking : l10n.vaultUnlock),
                ),
              ] else ...[
                if (store != null) ...[
                  VaultNotesView(key: ObjectKey(store), store: store),
                  const SizedBox(height: 24),
                  if (lock is VaultOpen && isReal) const VaultChangePinView(),
                  const SizedBox(height: 16),
                ],
                FilledButton.tonal(
                  onPressed: () =>
                      ref.read(vaultControllerProvider.notifier).lock(),
                  child: Text(l10n.vaultLockAction),
                ),
              ],
            ],
          ],
        ),
      ),
    );
  }
}

final class _PinField extends StatelessWidget {
  const _PinField({
    required this.controller,
    required this.hint,
    required this.enabled,
    this.onSubmitted,
  });

  final TextEditingController controller;
  final String hint;
  final bool enabled;
  final ValueChanged<String>? onSubmitted;

  @override
  Widget build(BuildContext context) => TextField(
    controller: controller,
    obscureText: true,
    keyboardType: TextInputType.number,
    inputFormatters: [FilteringTextInputFormatter.digitsOnly],
    autocorrect: false,
    enableSuggestions: false,
    enableIMEPersonalizedLearning: false,
    onSubmitted: onSubmitted,
    enabled: enabled,
    decoration: InputDecoration(hintText: hint),
  );
}

/// `m:ss`, or `h:mm:ss` from one hour up. Digits only, so it needs no
/// translation.
String _formatWait(Duration d) {
  final total = d.inSeconds < 1 ? 1 : d.inSeconds;
  final h = total ~/ 3600;
  final m = (total % 3600) ~/ 60;
  final sec = (total % 60).toString().padLeft(2, '0');
  return h > 0
      ? '$h:${m.toString().padLeft(2, '0')}:$sec'
      : '$m:$sec';
}
