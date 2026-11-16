import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:sargoshi/main.dart';

void main() {
  testWidgets('app starts and shows the localized wordmark and demo warning', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(412, 915));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(const ProviderScope(child: SargoshiApp()));
    await tester.pumpAndSettle();
    expect(find.text('Sargoshi'), findsOneWidget);
    expect(find.text('DEMO CRYPTO — NOT SECURE'), findsOneWidget);
    expect(find.text('Maya'), findsOneWidget);
  });

  testWidgets(
    'demo thread holds a draft during key review and releases it on accept',
    (tester) async {
      await tester.binding.setSurfaceSize(const Size(412, 915));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      await tester.pumpWidget(const ProviderScope(child: SargoshiApp()));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Maya'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Simulate change'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(TextField).last, 'Held local draft');
      await tester.testTextInput.receiveAction(TextInputAction.send);
      await tester.pumpAndSettle();
      expect(find.text('Held local draft'), findsNothing);
      expect(find.textContaining('1 outgoing message'), findsOneWidget);
      await tester.tap(find.text('Accept key change in demo'));
      await tester.pumpAndSettle();
      expect(find.text('Held local draft'), findsOneWidget);
      expect(
        find.textContaining('identity remains unverified'),
        findsOneWidget,
      );
    },
  );
}
