import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:sargoshi/main.dart';

import '../helpers/screen.dart';

const _banner = 'DEMO CRYPTO — NOT SECURE';

void main() {
  Future<void> openVault(WidgetTester tester) async {
    useScreen(tester);
    await tester.pumpWidget(const ProviderScope(child: SargoshiApp()));
    await tester.pumpAndSettle();
    await tester.longPress(find.text('Sargoshi'));
    await tester.pumpAndSettle();
  }

  testWidgets(
    'long-pressing the wordmark opens a vault that says it is a demo',
    (tester) async {
      await openVault(tester);
      expect(find.text('Vault locked'), findsOneWidget);
      expect(find.textContaining('nothing here is protected'), findsOneWidget);
    },
  );

  testWidgets('the demo banner is still visible on the vault screen', (
    tester,
  ) async {
    await openVault(tester);
    expect(find.text(_banner), findsOneWidget);
  });

  testWidgets('wrong PIN shows the failed-attempt count and stays locked', (
    tester,
  ) async {
    await openVault(tester);
    await tester.enterText(find.byType(TextField), '9999');
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    expect(find.text('Incorrect PIN. Failed attempts: 1'), findsOneWidget);
    expect(find.text('Vault locked'), findsOneWidget);
  });

  testWidgets('a clear error does not linger after the vault opens', (
    tester,
  ) async {
    await openVault(tester);
    await tester.enterText(find.byType(TextField), '9999');
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), '1234');
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    expect(find.text('Demo vault open'), findsOneWidget);
    expect(find.textContaining('Incorrect PIN'), findsNothing);
  });

  testWidgets('going back locks the vault again', (tester) async {
    await openVault(tester);
    await tester.enterText(find.byType(TextField), '1234');
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    expect(find.text('Demo vault open'), findsOneWidget);
    final nav = tester.state<NavigatorState>(find.byType(Navigator).first);
    nav.pop();
    await tester.pumpAndSettle();
    await tester.longPress(find.text('Sargoshi'));
    await tester.pumpAndSettle();
    expect(find.text('Vault locked'), findsOneWidget);
  });
}
