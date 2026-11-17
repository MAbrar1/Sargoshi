import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:sargoshi/main.dart';

import '../helpers/screen.dart';

/// Spec section 5.6 and 5.2: smallest phone 320x568, 200% text, tablets.
void main() {
  Future<void> start(WidgetTester tester) async {
    await tester.pumpWidget(const ProviderScope(child: SargoshiApp()));
    await tester.pumpAndSettle();
  }

  for (final scale in [1.0, 1.3, 2.0]) {
    testWidgets('chat list has no overflow at 320x568, text x$scale', (
      tester,
    ) async {
      useScreen(tester, size: const Size(320, 568), textScale: scale);
      await start(tester);
      expect(tester.takeException(), isNull);
    });

    testWidgets('chat thread has no overflow at 320x568, text x$scale', (
      tester,
    ) async {
      useScreen(tester, size: const Size(320, 568), textScale: scale);
      await start(tester);
      await tester.scrollUntilVisible(
        find.text('Maya'),
        100,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.text('Maya'));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
    });

    testWidgets('vault has no overflow at 320x568, text x$scale', (
      tester,
    ) async {
      useScreen(tester, size: const Size(320, 568), textScale: scale);
      await start(tester);
      await tester.longPress(find.text('Sargoshi'));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
    });
  }

  testWidgets('tablet width shows two panes', (tester) async {
    useScreen(tester, size: const Size(900, 700));
    await start(tester);
    await tester.tap(find.text('Maya'));
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(find.text('Omar'), findsOneWidget);
  });
}
