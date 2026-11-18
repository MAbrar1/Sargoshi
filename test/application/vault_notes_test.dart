import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/vault/vault_notes.dart';

import '../helpers/fake_vault_store.dart';

void main() {
  late FakeVaultStore store;
  late VaultNotes notes;

  setUp(() {
    store = FakeVaultStore();
    notes = VaultNotes(store);
  });

  test('starts empty', () async {
    expect(await notes.all(), isEmpty);
  });

  test('adds notes oldest first and keeps non-ASCII text', () async {
    await notes.add('first');
    await notes.add('نرمی سے کہو');
    await notes.add('third');
    final all = await notes.all();
    expect(all.map((n) => n.text), ['first', 'نرمی سے کہو', 'third']);
    expect(all.map((n) => n.id).toSet().length, 3);
  });

  test('notes are stored as items of kind note, not plain text keys', () async {
    final note = await notes.add('hello');
    expect(store.data.keys, ['note/${note.id}']);
  });

  test('ids are valid vault ids', () async {
    final note = await notes.add('x');
    expect(RegExp(r'^[A-Za-z0-9_-]{1,64}$').hasMatch(note.id), isTrue);
  });

  test('remove deletes only that note', () async {
    final a = await notes.add('a');
    await notes.add('b');
    await notes.remove(a.id);
    expect((await notes.all()).map((n) => n.text), ['b']);
  });

  test('malformed stored bytes do not crash reading', () async {
    await store.put(kind: 'note', id: 'bad', plain: Uint8List.fromList([0xff]));
    expect(await notes.all(), hasLength(1));
  });

  test('a locked store fails instead of returning stale notes', () async {
    await notes.add('a');
    store.lock();
    await expectLater(notes.all(), throwsA(isA<Exception>()));
  });
}
