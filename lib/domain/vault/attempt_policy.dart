/// Wrong PINs allowed before any delay applies.
const int kFreeAttempts = 4;

/// Delay before the next attempt after [failures] consecutive wrong PINs.
///
/// 5 wrong: 30 s, 6: 1 min, 7: 5 min, 8: 15 min, 9 or more: 1 hour.
/// A success resets the count. Nothing here wipes data: erasing the vault
/// after repeated failures is an owner decision that is not made yet.
Duration delayAfter(int failures) {
  if (failures <= kFreeAttempts) return Duration.zero;
  return switch (failures) {
    5 => const Duration(seconds: 30),
    6 => const Duration(minutes: 1),
    7 => const Duration(minutes: 5),
    8 => const Duration(minutes: 15),
    _ => const Duration(hours: 1),
  };
}
