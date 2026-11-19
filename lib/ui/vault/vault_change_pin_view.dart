import 'package:flutter/services.dart' show FilteringTextInputFormatter;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../application/vault/vault_pin_change.dart';
import '../../l10n/app_localizations.dart';

/// Change the real vault PIN. Shown only while the real vault is open.
final class VaultChangePinView extends ConsumerStatefulWidget {
  const VaultChangePinView({super.key});

  @override
  ConsumerState<VaultChangePinView> createState() => _VaultChangePinViewState();
}

class _VaultChangePinViewState extends ConsumerState<VaultChangePinView> {
  final _current = TextEditingController();
  final _next = TextEditingController();
  final _confirm = TextEditingController();
  bool _busy = false;
  String? _message;
  bool _success = false;

  @override
  void dispose() {
    _current.dispose();
    _next.dispose();
    _confirm.dispose();
    super.dispose();
  }

  Future<void> _submit(AppLocalizations l10n) async {
    if (_busy) return;
    if (_next.text != _confirm.text) {
      setState(() {
        _success = false;
        _message = l10n.vaultPinMismatch;
      });
      return;
    }
    setState(() {
      _busy = true;
      _message = null;
    });
    final result = await ref
        .read(vaultPinChangeProvider)
        .change(currentPin: _current.text, newPin: _next.text);
    if (!mounted) return;
    if (result == PinChangeResult.changed ||
        result == PinChangeResult.incomplete) {
      _current.clear();
      _next.clear();
      _confirm.clear();
    }
    setState(() {
      _busy = false;
      _success = result == PinChangeResult.changed;
      _message = switch (result) {
        PinChangeResult.changed => l10n.vaultPinChanged,
        PinChangeResult.wrongCurrent => l10n.vaultPinWrongCurrent,
        PinChangeResult.tooShort => l10n.vaultPinTooShort,
        PinChangeResult.sameAsCurrent => l10n.vaultPinSame,
        PinChangeResult.clashesWithDecoy => l10n.vaultPinClash,
        PinChangeResult.incomplete => l10n.vaultPinChangeIncomplete,
        PinChangeResult.failed => l10n.vaultPinChangeFailed,
      };
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final theme = Theme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(l10n.vaultChangePinTitle, style: theme.textTheme.titleMedium),
        const SizedBox(height: 8),
        _field(_current, l10n.vaultCurrentPinHint),
        const SizedBox(height: 8),
        _field(_next, l10n.vaultNewPinHint),
        const SizedBox(height: 8),
        _field(
          _confirm,
          l10n.vaultPinConfirmHint,
          onSubmitted: (_) => _submit(l10n),
        ),
        if (_message != null) ...[
          const SizedBox(height: 8),
          Text(
            _message!,
            style: _success ? null : TextStyle(color: theme.colorScheme.error),
          ),
        ],
        const SizedBox(height: 8),
        FilledButton(
          onPressed: _busy ? null : () => _submit(l10n),
          child: Text(_busy ? l10n.vaultChecking : l10n.vaultChangePinAction),
        ),
      ],
    );
  }

  Widget _field(
    TextEditingController controller,
    String hint, {
    ValueChanged<String>? onSubmitted,
  }) => TextField(
    controller: controller,
    enabled: !_busy,
    obscureText: true,
    keyboardType: TextInputType.number,
    inputFormatters: [FilteringTextInputFormatter.digitsOnly],
    autocorrect: false,
    enableSuggestions: false,
    enableIMEPersonalizedLearning: false,
    onSubmitted: onSubmitted,
    decoration: InputDecoration(hintText: hint),
  );
}
