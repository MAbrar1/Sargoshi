import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../../application/vault/vault_content.dart';
import '../../application/vault/vault_notes.dart';
import '../../l10n/app_localizations.dart';

/// Notes kept in the unlocked vault. Everything is read from and written to the
/// encrypted store; nothing is cached outside this widget's state.
final class VaultNotesView extends StatefulWidget {
  const VaultNotesView({required this.store, super.key});

  final VaultItemStore store;

  @override
  State<VaultNotesView> createState() => _VaultNotesViewState();
}

class _VaultNotesViewState extends State<VaultNotesView> {
  late final VaultNotes _notes = VaultNotes(widget.store);
  final _text = TextEditingController();
  List<VaultNote> _items = const <VaultNote>[];
  bool _loaded = false;
  bool _failed = false;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _busy = true;
    unawaited(
      _run(() async {
        _items = await _notes.all();
      }, markBusy: false),
    );
  }

  @override
  void dispose() {
    _text.dispose();
    super.dispose();
  }

  /// Runs [action], then marks the list loaded or failed. Errors are never
  /// shown raw.
  Future<void> _run(
    Future<void> Function() action, {
    bool markBusy = true,
  }) async {
    if (markBusy) setState(() => _busy = true);
    var failed = false;
    try {
      await action();
    } on Object {
      failed = true;
    }
    if (!mounted) return;
    setState(() {
      _busy = false;
      _loaded = true;
      _failed = failed;
    });
  }

  Future<void> _add() async {
    final text = _text.text.trim();
    if (text.isEmpty || _busy) return;
    await _run(() async {
      final note = await _notes.add(text);
      _items = [..._items, note];
      _text.clear();
    });
  }

  Future<void> _remove(VaultNote note) async {
    if (_busy) return;
    await _run(() async {
      await _notes.remove(note.id);
      _items = _items.where((n) => n.id != note.id).toList();
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final theme = Theme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(l10n.vaultNotesTitle, style: theme.textTheme.titleMedium),
        const SizedBox(height: 8),
        if (_failed)
          Text(
            l10n.vaultNotesFailed,
            style: TextStyle(color: theme.colorScheme.error),
          )
        else if (_loaded && _items.isEmpty)
          Text(l10n.vaultNotesEmpty),
        for (final note in _items)
          Padding(
            padding: const EdgeInsetsDirectional.only(bottom: 8),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(child: Text(note.text)),
                TextButton(
                  onPressed: _busy ? null : () => _remove(note),
                  child: Text(l10n.vaultNoteDelete),
                ),
              ],
            ),
          ),
        const SizedBox(height: 8),
        TextField(
          controller: _text,
          enabled: !_busy,
          minLines: 1,
          maxLines: 4,
          autocorrect: false,
          enableSuggestions: false,
          enableIMEPersonalizedLearning: false,
          decoration: InputDecoration(hintText: l10n.vaultNoteHint),
        ),
        const SizedBox(height: 8),
        FilledButton(
          onPressed: _busy ? null : _add,
          child: Text(l10n.vaultNoteAdd),
        ),
      ],
    );
  }
}
