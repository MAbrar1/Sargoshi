import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/measured/measured.dart';
import '../../domain/trust/trust_state.dart';

final trustProvider = StateNotifierProvider<TrustController, TrustState>((ref) {
  final now = DateTime.now().toUtc();
  return TrustController(
    initialMeasurement: Measured.unknown(
      source: 'Manual trust review',
      measuredAt: now,
    ),
  );
});

final class TrustController extends StateNotifier<TrustState> {
  TrustController({required Measured<TrustValue> initialMeasurement})
      : super(
          TrustState(
            measurement: initialMeasurement,
            gateState: KeyChangeGateState.idle,
            reviewReason: null,
          ),
        );

  void markUnknown() {
    final now = DateTime.now().toUtc();
    state = state.copyWith(
      measurement: Measured.unknown(
        source: 'Manual trust review',
        measuredAt: now,
      ),
      gateState: KeyChangeGateState.idle,
      reviewReason: null,
    );
  }

  void markVerified() {
    final now = DateTime.now().toUtc();
    state = state.copyWith(
      measurement: Measured.of(
        TrustValue.verified,
        source: 'Manual trust review',
        measuredAt: now,
      ),
      gateState: KeyChangeGateState.accepted,
      reviewReason: null,
    );
  }

  void reportKeyChanged() {
    final now = DateTime.now().toUtc();
    state = state.copyWith(
      measurement: Measured.of(
        TrustValue.keyChanged,
        source: 'Key change observed',
        measuredAt: now,
      ),
      gateState: KeyChangeGateState.awaitingReview,
      reviewReason: 'A key change was observed and sending is paused.',
    );
  }

  void acceptKeyChange() {
    final now = DateTime.now().toUtc();
    state = state.copyWith(
      measurement: Measured.of(
        TrustValue.verified,
        source: 'User accepted key change',
        measuredAt: now,
      ),
      gateState: KeyChangeGateState.accepted,
      reviewReason: null,
    );
  }

  void blockKeyChange() {
    final now = DateTime.now().toUtc();
    state = state.copyWith(
      measurement: Measured.of(
        TrustValue.keyChanged,
        source: 'User blocked key change',
        measuredAt: now,
      ),
      gateState: KeyChangeGateState.blocked,
      reviewReason: 'Sending remains blocked until the user reviews the change.',
    );
  }
}
