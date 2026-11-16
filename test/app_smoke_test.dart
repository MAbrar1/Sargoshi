import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/main.dart';

void main() {
  testWidgets('app starts and shows the localized wordmark', (tester) async {
    await tester.pumpWidget(const ProviderScope(child: SargoshiApp()));
    await tester.pumpAndSettle();
    expect(find.text('Sargoshi'), findsOneWidget);
  });
}
