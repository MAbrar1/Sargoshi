import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/main.dart';

import '../helpers/screen.dart';

void main() {
  Future<void> openTrust(WidgetTester tester) async {
    useScreen(tester);
    await tester.pumpWidget(const ProviderScope(child: SargoshiApp()));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Maya'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Trust details'));
    await tester.pumpAndSettle();
  }

  testWidgets('trust screen shows Unknown with no evidence, never Verified', (
    tester,
  ) async {
    await openTrust(tester);
    expect(find.text('Unknown'), findsOneWidget);
    expect(find.text('Verified'), findsNothing);
    expect(
      find.text('No protocol evidence is available in this demo.'),
      findsOneWidget,
    );
  });

  testWidgets('the demo banner is visible on the trust screen', (tester) async {
    await openTrust(tester);
    expect(find.text('DEMO CRYPTO — NOT SECURE'), findsOneWidget);
  });

  testWidgets('accepting a key change does not make the contact Verified', (
    tester,
  ) async {
    await openTrust(tester);
    await tester.tap(find.text('Simulate key change (demo)'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Accept key change in demo'));
    await tester.pumpAndSettle();
    expect(find.text('Verified'), findsNothing);
    expect(find.text('Unknown'), findsOneWidget);
    expect(
      find.text('Accepted for this demo session. Identity remains unverified.'),
      findsOneWidget,
    );
  });
}
