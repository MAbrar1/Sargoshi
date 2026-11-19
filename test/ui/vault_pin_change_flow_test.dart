import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:sargoshi/application/vault/vault_backend.dart';
import 'package:sargoshi/application/vault/vault_content.dart';
import 'package:sargoshi/application/vault/vault_session.dart';
import 'package:sargoshi/domain/vault/vault_lock.dart';
import 'package:sargoshi/main.dart';

import '../helpers/fake_vault_store.dart';
import '../helpers/screen.dart';

final class _Backend implements VaultBackend {
  String pin = '2468';
  static const decoy = '0000';

  @override
  PinVerifier get verifier => _Verifier(this);

  @override
  Future<bool> isSetUp() async => true;

  @override
  Future<void> setPins({required String pin, String? decoyPin}) async {}

  @override
  Future<void> changeRealPin(String pin) async => this.pin = pin;
}

final class _Verifier implements PinVerifier {
  const _Verifier(this.backend);
  final _Backend backend;

  @override
  Future<PinCheck> check(String pin) async {
    if (pin == backend.pin) return PinCheck.accepted;
    if (pin == _Backend.decoy) return PinCheck.decoy;
    return PinCheck.rejected;
  }
}

Future<void> reveal(WidgetTester tester, Finder finder) async {
  if (finder.hitTestable().evaluate().isNotEmpty) return;
  await tester.scrollUntilVisible(
    finder,
    100,
    scrollable: find.byType(Scrollable).first,
  );
}

Finder field(String hint) => find.byWidgetPredicate(
  (w) => w is TextField && w.decoration?.hintText == hint,
);

void main() {
  late _Backend backend;
  late FakeOpener opener;

  Future<void> unlock(
    WidgetTester tester, {
    String pin = '2468',
    Size size = const Size(412, 915),
    double textScale = 1.0,
  }) async {
    useScreen(tester, size: size, textScale: textScale);
    backend = _Backend();
    opener = FakeOpener();
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          vaultBackendProvider.overrideWithValue(backend),
          vaultContentOpenerProvider.overrideWithValue(opener),
        ],
        child: const SargoshiApp(),
      ),
    );
    await tester.pumpAndSettle();
    await tester.longPress(find.text('Sargoshi'));
    await tester.pumpAndSettle();
    await reveal(tester, find.byType(TextField));
    await tester.enterText(find.byType(TextField), pin);
    await reveal(tester, find.text('Unlock'));
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
  }

  Future<void> change(
    WidgetTester tester, {
    String current = '2468',
    String next = '8642',
    String? confirm,
  }) async {
    for (final (hint, text) in [
      ('Current PIN', current),
      ('New PIN', next),
      ('Enter the PIN again', confirm ?? next),
    ]) {
      await reveal(tester, field(hint));
      await tester.enterText(field(hint), text);
    }
    await reveal(tester, find.widgetWithText(FilledButton, 'Change PIN'));
    await tester.tap(find.widgetWithText(FilledButton, 'Change PIN'));
    await tester.pumpAndSettle();
  }

  testWidgets('a successful change re-keys the vault and keeps notes', (
    tester,
  ) async {
    await unlock(tester);
    await change(tester);
    await reveal(tester, find.text('PIN changed. Your notes were kept.'));
    expect(find.text('PIN changed. Your notes were kept.'), findsOneWidget);
    expect(backend.pin, '8642');
    expect(opener.changes, [(oldPin: '2468', newPin: '8642')]);
    expect(find.text('Notes'), findsOneWidget);
  });

  testWidgets('the new PIN opens the vault after locking, the old one does not', (
    tester,
  ) async {
    await unlock(tester);
    await change(tester);
    await reveal(tester, find.text('Lock vault'));
    await tester.tap(find.text('Lock vault'));
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextField), '2468');
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    expect(find.textContaining('Incorrect PIN'), findsOneWidget);

    await tester.enterText(find.byType(TextField), '8642');
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    expect(find.text('Notes'), findsOneWidget);
  });

  testWidgets('a wrong current PIN changes nothing', (tester) async {
    await unlock(tester);
    await change(tester, current: '1111');
    expect(
      find.text('The current PIN is not correct. Nothing was changed.'),
      findsOneWidget,
    );
    expect(backend.pin, '2468');
    expect(opener.changes, isEmpty);
  });

  testWidgets('mismatched, short and identical PINs are rejected locally', (
    tester,
  ) async {
    await unlock(tester);
    await change(tester, next: '8642', confirm: '8643');
    expect(find.text('The two PINs do not match.'), findsOneWidget);
    await change(tester, next: '12');
    expect(find.text('Use at least 4 digits.'), findsOneWidget);
    await change(tester, next: '2468');
    expect(
      find.text('The new PIN is the same as the current one.'),
      findsOneWidget,
    );
    expect(opener.changes, isEmpty);
    expect(backend.pin, '2468');
  });

  testWidgets('an unfinished change tells the truth', (tester) async {
    await unlock(tester);
    opener.changeFailure = const VaultPinChangeIncompleteException();
    await change(tester);
    expect(find.textContaining('may not have finished'), findsOneWidget);
  });

  testWidgets('a plain failure says nothing was changed', (tester) async {
    await unlock(tester);
    opener.changeFailure = StateError('disk');
    await change(tester);
    expect(
      find.text('The PIN could not be changed. Nothing was changed.'),
      findsOneWidget,
    );
  });

  testWidgets('the decoy vault has no PIN-change form', (tester) async {
    await unlock(tester, pin: _Backend.decoy);
    expect(find.text('Notes'), findsOneWidget);
    expect(find.text('Change PIN', skipOffstage: false), findsNothing);
  });

  for (final scale in [1.0, 2.0]) {
    testWidgets('change form has no overflow at 320x568, text x$scale', (
      tester,
    ) async {
      await unlock(tester, size: const Size(320, 568), textScale: scale);
      await reveal(tester, field('Current PIN'));
      expect(tester.takeException(), isNull);
      expect(field('Current PIN'), findsOneWidget);
    });
  }
}
