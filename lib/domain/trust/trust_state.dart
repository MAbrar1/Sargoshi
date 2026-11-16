import 'package:meta/meta.dart';

import '../../core/measured/measured.dart';

/// Honest trust status values from measured evidence. 'Unknown' is the default
/// when no verified evidence exists, never a green state.
enum TrustValue { unknown, verified, keyChanged }

enum KeyChangeGateState { idle, awaitingReview, accepted, blocked }

@immutable
final class TrustState {
  const TrustState({
    required this.measurement,
    required this.gateState,
    required this.reviewReason,
  });

  final Measured<TrustValue> measurement;
  final KeyChangeGateState gateState;
  final String? reviewReason;

  bool get isAwaitingReview => gateState == KeyChangeGateState.awaitingReview;

  TrustState copyWith({
    Measured<TrustValue>? measurement,
    KeyChangeGateState? gateState,
    String? reviewReason,
  }) => TrustState(
    measurement: measurement ?? this.measurement,
    gateState: gateState ?? this.gateState,
    reviewReason: reviewReason ?? this.reviewReason,
  );
}
