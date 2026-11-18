import 'dart:convert';
import 'dart:typed_data';

import 'vault_content.dart';

/// One note kept in the vault.
final class VaultNote {
  const VaultNote({required this.id, required this.text});

  final String id;
  final String text;
}

/// Notes stored as sealed vault items of kind `note`.
final class VaultNotes {
  VaultNotes(this.store);

  final VaultItemStore store;

  static const kind = 'note';

  int _counter = 0;

  /// All notes, oldest first (ids sort by creation time).
  Future<List<VaultNote>> all() async {
    final notes = <VaultNote>[];
    for (final id in await store.list(kind)) {
      final bytes = await store.get(kind: kind, id: id);
      notes.add(
        VaultNote(id: id, text: utf8.decode(bytes, allowMalformed: true)),
      );
    }
    return notes;
  }

  Future<VaultNote> add(String text) async {
    final id = _newId();
    await store.put(
      kind: kind,
      id: id,
      plain: Uint8List.fromList(utf8.encode(text)),
    );
    return VaultNote(id: id, text: text);
  }

  Future<void> remove(String id) => store.delete(kind: kind, id: id);

  String _newId() {
    final micros = DateTime.now().microsecondsSinceEpoch
        .toRadixString(36)
        .padLeft(12, '0');
    final count = (_counter++).toRadixString(36).padLeft(4, '0');
    return '$micros-$count';
  }
}
