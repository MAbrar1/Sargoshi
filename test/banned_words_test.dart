import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// Spec section 0: banned claims must never appear in user-visible strings,
/// comments, store text or test names. The list lives in tool/ci/banned_words.txt
/// so the Python CI check and this test share one source.
void main() {
  final banned = File('tool/ci/banned_words.txt')
      .readAsLinesSync()
      .map((l) => l.trim().toLowerCase())
      .where((l) => l.isNotEmpty)
      .toList();

  test('banned word list is not empty', () {
    expect(banned, isNotEmpty);
  });

  test('no banned words in lib, ARB, tests or Android resources', () {
    const exts = {'.dart', '.arb', '.xml', '.kt', '.md'};
    final hits = <String>[];
    for (final dir in ['lib', 'test', 'android/app/src', 'docs']) {
      final d = Directory(dir);
      if (!d.existsSync()) continue;
      for (final f in d.listSync(recursive: true).whereType<File>()) {
        final dot = f.path.lastIndexOf('.');
        if (dot < 0 || !exts.contains(f.path.substring(dot))) continue;
        final text = f.readAsStringSync().toLowerCase();
        for (final w in banned) {
          if (text.contains(w)) hits.add('${f.path}: $w');
        }
      }
    }
    expect(hits, isEmpty);
  });
}
