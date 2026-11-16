import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/application/trust/trust_controller.dart';
import 'package:sargoshi/core/measured/measured.dart';
import 'package:sargoshi/domain/trust/trust_state.dart';

void main() {
  final now = DateTime.utc(2026, 1, 1);

  TrustController createController() => TrustController(
    initialMeasurement: Measured.unknown(
      source: kDemoTrustSource,
      measuredAt: now,
    ),
  );

  test('unknown measurement never implies verified identity', () {
    final controller = createController();
    addTearDown(controller.dispose);

    expect(controller.state.measurement.value, isNull);
    expect(controller.state.canSend, isTrue);
  });

  test('simulated key change holds sends pending explicit review', () {
    final controller = createController();
    addTearDown(controller.dispose);

    controller.simulateKeyChange();

    expect(controller.state.gateState, KeyChangeGateState.awaitingReview);
    expect(controller.state.canSend, isFalse);
    expect(controller.state.measurement.value, isNull);
  });

  test(
    'accepting a change releases gate but does not mark identity verified',
    () {
      final controller = createController();
      addTearDown(controller.dispose);
      controller.simulateKeyChange();

      controller.acceptKeyChange();

      expect(controller.state.gateState, KeyChangeGateState.accepted);
      expect(controller.state.canSend, isTrue);
      expect(controller.state.measurement.value, isNull);
    },
  );

  test('blocking a change keeps sending disabled', () {
    final controller = createController();
    addTearDown(controller.dispose);
    controller.simulateKeyChange();

    controller.blockKeyChange();

    expect(controller.state.gateState, KeyChangeGateState.blocked);
    expect(controller.state.canSend, isFalse);
  });

  test('review actions do nothing unless a change is pending', () {
    final controller = createController();
    addTearDown(controller.dispose);

    controller.acceptKeyChange();
    expect(controller.state.gateState, KeyChangeGateState.idle);
    controller.blockKeyChange();
    expect(controller.state.gateState, KeyChangeGateState.idle);
  });
}
