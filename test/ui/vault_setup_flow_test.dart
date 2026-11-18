import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:sargoshi/application/vault/vault_backend.dart';
import 'package:sargoshi/application/vault/vault_controller.dart';
import 'package:sargoshi/application/vault/vault_setup.dart';
import 'package:sargoshi/domain/vault/vault_lock.dart';
import 'package:sargoshi/main.dart';

import '../helpers/screen.dart';

final class _FakeBackend implements VaultBackend {
  _FakeBackend({this.fail = false});

  final bool fail;
  bool setUp = false;
  String? pin;

  @override
  PinVerifier get verifier => _Verifier(this);

  @override
  Future<bool> isSetUp() async => setUp;

  @override
  Future<void> setPins({required String pin, String? decoyPin}) async {
    if (fail) throw StateError('platform down');
    if (pin.length < kMinPinLength) {
      throw const VaultSetupException(VaultSetupError.tooShort);
    }
    this.pin = pin;
    setUp = true;
  }
}

final class _Verifier implements PinVerifier {
  const _Verifier(this.backend);
  final _FakeBackend backend;

  @override
  Future<PinCheck> check(String pin) async =>
      pin == backend.pin ? PinCheck.accepted : PinCheck.rejected;
}

void main() {
  Future<_FakeBackend> openVault(
    WidgetTester tester, {
    bool fail = false,
    double textScale = 1.0,
    Size size = const Size(412, 915),
  }) async {
    useScreen(tester, size: size, textScale: textScale);
    final backend = _FakeBackend(fail: fail);
    await tester.pumpWidget(
      ProviderScope(
        overrides: [vaultBackendProvider.overrideWithValue(backend)],
        child: const SargoshiApp(),
      ),
    );
    await tester.pumpAndSettle();
    await tester.longPress(find.text('Sargoshi'));
    await tester.pumpAndSettle();
    return backend;
  }

  Future<void> fill(WidgetTester tester, String first, String second) async {
    final fields = find.byType(TextField);
    await tester.enterText(fields.at(0), first);
    await tester.enterText(fields.at(1), second);
  }

  testWidgets('first use asks for a PIN and says what is real', (tester) async {
    await openVault(tester);
    expect(find.text('Set a vault PIN'), findsOneWidget);
    expect(find.textContaining('holds nothing yet'), findsOneWidget);
    expect(find.textContaining('Demo PIN: 1234'), findsNothing);
  });

  testWidgets('mismatched PINs are rejected and nothing is saved', (
    tester,
  ) async {
    final backend = await openVault(tester);
    await fill(tester, '2468', '2469');
    await tester.tap(find.text('Set PIN'));
    await tester.pumpAndSettle();
    expect(find.text('The two PINs do not match.'), findsOneWidget);
    expect(backend.setUp, isFalse);
  });

  testWidgets('a short PIN is rejected', (tester) async {
    final backend = await openVault(tester);
    await fill(tester, '12', '12');
    await tester.tap(find.text('Set PIN'));
    await tester.pumpAndSettle();
    expect(find.text('Use at least 4 digits.'), findsOneWidget);
    expect(backend.setUp, isFalse);
  });

  testWidgets('a platform failure says nothing was changed', (tester) async {
    final backend = await openVault(tester, fail: true);
    await fill(tester, '2468', '2468');
    await tester.tap(find.text('Set PIN'));
    await tester.pumpAndSettle();
    expect(find.textContaining('Nothing was changed'), findsOneWidget);
    expect(backend.setUp, isFalse);
    expect(find.text('Set a vault PIN'), findsOneWidget);
  });

  testWidgets('setting a PIN moves to unlock, and the PIN then opens it', (
    tester,
  ) async {
    await openVault(tester);
    await fill(tester, '2468', '2468');
    await tester.tap(find.text('Set PIN'));
    await tester.pumpAndSettle();
    expect(find.text('Vault locked'), findsOneWidget);
    await tester.enterText(find.byType(TextField), '2468');
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    expect(find.text('Demo vault open'), findsOneWidget);
  });

  testWidgets('wrong PIN is rejected after setup', (tester) async {
    await openVault(tester);
    await fill(tester, '2468', '2468');
    await tester.tap(find.text('Set PIN'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), '1111');
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    expect(find.text('Incorrect PIN. Failed attempts: 1'), findsOneWidget);
  });

  for (final scale in [1.0, 2.0]) {
    testWidgets('setup screen has no overflow at 320x568, text x$scale', (
      tester,
    ) async {
      await openVault(tester, size: const Size(320, 568), textScale: scale);
      expect(tester.takeException(), isNull);
      expect(find.text('Set a vault PIN'), findsOneWidget);
    });
  }
}
