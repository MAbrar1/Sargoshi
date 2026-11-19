import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:sargoshi/application/vault/vault_backend.dart';
import 'package:sargoshi/application/vault/vault_session.dart';
import 'package:sargoshi/domain/vault/vault_lock.dart';
import 'package:sargoshi/main.dart';

import '../helpers/fake_vault_store.dart';
import '../helpers/screen.dart';

final class _Backend implements VaultBackend {
  @override
  PinVerifier get verifier => _Verifier();

  @override
  Future<bool> isSetUp() async => true;

  @override
  Future<void> setPins({required String pin, String? decoyPin}) async {}
}

final class _Verifier implements PinVerifier {
  @override
  Future<PinCheck> check(String pin) async =>
      pin == '2468' ? PinCheck.accepted : PinCheck.rejected;
}

/// At large text on a small screen the lazy list has not built everything yet.
Future<void> reveal(WidgetTester tester, Finder finder) async {
  if (finder.evaluate().isNotEmpty) return;
  await tester.scrollUntilVisible(
    finder,
    100,
    scrollable: find.byType(Scrollable).first,
  );
}

void main() {
  Future<FakeOpener> unlock(
    WidgetTester tester, {
    bool fail = false,
    Map<String, Uint8List> seed = const <String, Uint8List>{},
    double textScale = 1.0,
    Size size = const Size(412, 915),
  }) async {
    useScreen(tester, size: size, textScale: textScale);
    final opener = FakeOpener(fail: fail, seed: seed);
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          vaultBackendProvider.overrideWithValue(_Backend()),
          vaultContentOpenerProvider.overrideWithValue(opener),
        ],
        child: const SargoshiApp(),
      ),
    );
    await tester.pumpAndSettle();
    await tester.longPress(find.text('Sargoshi'));
    await tester.pumpAndSettle();
    await reveal(tester, find.byType(TextField));
    await tester.enterText(find.byType(TextField), '2468');
    await reveal(tester, find.text('Unlock'));
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    return opener;
  }

  testWidgets('says the vault is encrypted and what is not stored yet', (
    tester,
  ) async {
    await unlock(tester);
    expect(find.textContaining('encrypted on this phone'), findsOneWidget);
    expect(find.textContaining('independent review'), findsOneWidget);
    expect(find.textContaining('holds nothing yet'), findsNothing);
  });

  testWidgets('unlocking shows an empty notes list', (tester) async {
    await unlock(tester);
    expect(find.text('Notes'), findsOneWidget);
    expect(find.text('No notes yet.'), findsOneWidget);
  });

  testWidgets('a saved note appears and is stored in the vault store', (
    tester,
  ) async {
    final opener = await unlock(tester);
    await tester.enterText(find.byType(TextField), 'my first note');
    await tester.tap(find.text('Save note'));
    await tester.pumpAndSettle();
    expect(find.text('my first note'), findsOneWidget);
    expect(find.text('No notes yet.'), findsNothing);
    expect(opener.stores.single.data, hasLength(1));
  });

  testWidgets('blank notes are not saved', (tester) async {
    final opener = await unlock(tester);
    await tester.enterText(find.byType(TextField), '   ');
    await tester.tap(find.text('Save note'));
    await tester.pumpAndSettle();
    expect(opener.stores.single.data, isEmpty);
  });

  testWidgets('deleting a note removes it', (tester) async {
    final opener = await unlock(tester);
    await tester.enterText(find.byType(TextField), 'temporary');
    await tester.tap(find.text('Save note'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete note'));
    await tester.pumpAndSettle();
    expect(find.text('temporary'), findsNothing);
    expect(find.text('No notes yet.'), findsOneWidget);
    expect(opener.stores.single.data, isEmpty);
  });

  testWidgets('locking hides the notes and overwrites the key', (tester) async {
    final opener = await unlock(tester);
    await tester.enterText(find.byType(TextField), 'secret');
    await tester.tap(find.text('Save note'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Lock vault'));
    await tester.pumpAndSettle();
    expect(find.text('secret'), findsNothing);
    expect(find.text('Notes'), findsNothing);
    expect(find.text('Vault locked'), findsOneWidget);
    expect(opener.stores.single.locked, isTrue);
  });

  testWidgets('if content cannot open, nothing opens and it says so', (
    tester,
  ) async {
    await unlock(tester, fail: true);
    expect(find.text('Notes'), findsNothing);
    expect(find.text('Vault locked'), findsOneWidget);
    expect(find.textContaining('could not run'), findsOneWidget);
  });

  for (final scale in [1.0, 2.0]) {
    testWidgets('notes have no overflow at 320x568, text x$scale', (
      tester,
    ) async {
      await unlock(
        tester,
        size: const Size(320, 568),
        textScale: scale,
        seed: {'note/a': Uint8List.fromList(List.filled(120, 97))},
      );
      final err = tester.takeException();
      if (err is FlutterError) {
        // ignore: avoid_print
        print('DIAG ${err.diagnostics.map((n) => n.toStringDeep()).join('\n')}');
      }
      expect(err, isNull);
      expect(find.text('Notes', skipOffstage: false), findsOneWidget);
    });
  }
}
