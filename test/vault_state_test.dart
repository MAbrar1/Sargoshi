import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/domain/vault/vault_state.dart';

void main() {
  test('vault controller rejects short pins', () {
    final controller = VaultSessionController();
    controller.unlock('12', const AppLocalizationsTest());
    expect(controller.state.state, VaultLockState.error);
  });

  test('vault controller opens when pin is long enough', () {
    final controller = VaultSessionController();
    controller.unlock('1234', const AppLocalizationsTest());
    expect(controller.state.state, VaultLockState.unlocked);
  });
}

class AppLocalizationsTest {
  const AppLocalizationsTest();

  String get vaultPinTooShort => 'PIN is too short. Use at least 4 digits.';
}
