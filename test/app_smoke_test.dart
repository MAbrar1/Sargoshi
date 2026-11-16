import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/main.dart';

void main() {
  testWidgets('demo app shows the chat list and permanent warning', (tester) async {
    await tester.binding.setSurfaceSize(const Size(412, 915));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(const ProviderScope(child: SargoshiApp()));
    await tester.pumpAndSettle();
    expect(find.text('Sargoshi'), findsOneWidget);
    expect(find.text('DEMO CRYPTO — NOT SECURE'), findsOneWidget);
    expect(find.text('Maya'), findsOneWidget);
  });

  testWidgets('demo thread adds local-only messages', (tester) async {
    await tester.binding.setSurfaceSize(const Size(412, 915));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(const ProviderScope(child: SargoshiApp()));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Maya'));
    await tester.pumpAndSettle();
    expect(find.text('Would you like to meet for tea this afternoon?'), findsOneWidget);
    await tester.enterText(find.byType(TextField).last, 'A local demo reply');
    await tester.tap(find.byTooltip('Add demo message'));
    await tester.pumpAndSettle();
    expect(find.text('A local demo reply'), findsOneWidget);
  });
}
