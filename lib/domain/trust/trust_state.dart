import '../../core/measured/measured.dart';

enum TrustValue { unverified, verified, keyChanged }

enum KeyChangeGateState { idle, awaitingReview, accepted, blocked }

/// The measurement describes evidence about identity. The gate separately
/// records the user's response to a key-change event; accepting a change does
/// not itself prove the contact's identity.
final class TrustState {
  const TrustState({required this.measurement, required this.gateState});

  final Measured<TrustValue> measurement;
  final KeyChangeGateState gateState;

  bool get isAwaitingReview => gateState == KeyChangeGateState.awaitingReview;

  /// Sending is held until an observed change is reviewed or blocked.
  /// This gate is domain state only; no network delivery is implemented here.
  bool get canSend =>
      gateState == KeyChangeGateState.idle ||
      gateState == KeyChangeGateState.accepted;

  TrustState copyWith({
    Measured<TrustValue>? measurement,
    KeyChangeGateState? gateState,
  }) => TrustState(
    measurement: measurement ?? this.measurement,
    gateState: gateState ?? this.gateState,
  );
}
