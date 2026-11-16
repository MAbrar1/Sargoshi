import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/measured/measured.dart';
import '../../domain/trust/trust_state.dart';

const String kDemoTrustSource = 'demo/manual-review';

final trustProvider =
    StateNotifierProvider.family<TrustController, TrustState, String>((
      ref,
      contactId,
    ) {
      final now = DateTime.now().toUtc();
      return TrustController(
        initialMeasurement: Measured.unknown(
          source: '$kDemoTrustSource/$contactId',
          measuredAt: now,
        ),
      );
    });

/// Demo-only gate controller. It does not read protocol key events or verify
/// safety numbers. Production must replace its event source before use.
final class TrustController extends StateNotifier<TrustState> {
  TrustController({required Measured<TrustValue> initialMeasurement})
    : super(
        TrustState(
          measurement: initialMeasurement,
          gateState: KeyChangeGateState.idle,
        ),
      );

  /// A test/demo trigger only; it is not evidence that a remote key changed.
  void simulateKeyChange() {
    state = state.copyWith(gateState: KeyChangeGateState.awaitingReview);
  }

  /// Acknowledgement permits the demo gate to proceed but does not turn the
  /// trust measurement into Verified.
  void acceptKeyChange() {
    if (!state.isAwaitingReview) return;
    state = state.copyWith(gateState: KeyChangeGateState.accepted);
  }

  void blockKeyChange() {
    if (!state.isAwaitingReview) return;
    state = state.copyWith(gateState: KeyChangeGateState.blocked);
  }
}
