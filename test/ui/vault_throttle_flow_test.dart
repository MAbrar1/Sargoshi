import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:sargoshi/application/vault/attempt_limiter.dart';
import 'package:sargoshi/application/vault/vault_controller.dart';
import 'package:sargoshi/main.dart';

import '../helpers/fake_attempts.dart';
import '../helpers/screen.dart';

void main() {
  late FakeAttemptClock clock;

  Future<void> openVault(
    WidgetTester tester, {
    Size size = const Size(412, 915),
    double textScale = 1.0,
  }) async {
    useScreen(tester, size: size, textScale: textScale);
    clock = FakeAttemptClock();
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          attemptLimiterProvider.overrideWithValue(
            AttemptLimiter(store: InMemoryAttemptStore(), clock: clock),
          ),
        ],
        child: const SargoshiApp(),
      ),
    );
    await tester.pumpAndSettle();
    await tester.longPress(find.text('Sargoshi'));
    await tester.pumpAndSettle();
  }

  Future<void> tryPin(WidgetTester tester, String pin) async {
    await tester.enterText(find.byType(TextField), pin);
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
  }

  Future<void> closeScreen(WidgetTester tester) =>
      tester.pumpWidget(const SizedBox());

  testWidgets('five wrong PINs show a delay and disable Unlock', (
    tester,
  ) async {
    await openVault(tester);
    for (var i = 0; i < 5; i++) {
      await tryPin(tester, '9999');
    }
    expect(find.text('Too many wrong PINs. Try again in 0:30.'), findsOneWidget);
    final button = tester.widget<FilledButton>(
      find.widgetWithText(FilledButton, 'Unlock'),
    );
    expect(button.onPressed, isNull);
    await closeScreen(tester);
  });

  testWidgets('the delay ends with the clock and the right PIN then works', (
    tester,
  ) async {
    await openVault(tester);
    for (var i = 0; i < 5; i++) {
      await tryPin(tester, '9999');
    }
    clock.advance(const Duration(seconds: 31));
    await tester.pump(const Duration(seconds: 1));
    await tester.pumpAndSettle();
    expect(find.textContaining('Too many wrong PINs'), findsNothing);
    await tryPin(tester, '1234');
    expect(find.text('Demo vault open'), findsOneWidget);
  });

  for (final scale in [1.0, 2.0]) {
    testWidgets('delay message has no overflow at 320x568, text x$scale', (
      tester,
    ) async {
      await openVault(tester, size: const Size(320, 568), textScale: scale);
      for (var i = 0; i < 5; i++) {
        await tryPin(tester, '9999');
      }
      expect(tester.takeException(), isNull);
      await closeScreen(tester);
    });
  }
}
