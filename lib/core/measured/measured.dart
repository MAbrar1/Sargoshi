/// A security or health fact together with where it came from and when.
///
/// Status widgets accept only [Measured]. The constructor is private and
/// every factory requires a non-empty [source], so a value without a source
/// cannot be built. Unavailable measurements use [Measured.unknown], which
/// status widgets must render as "Unknown", never as a green state.
final class Measured<T> {
  const Measured._({
    required this.value,
    required this.source,
    required this.measuredAt,
  });

  /// A real measurement. [source] names the API or check that produced it,
  /// for example `KeyInfo.securityLevel`.
  factory Measured.of(
    T value, {
    required String source,
    required DateTime measuredAt,
  }) {
    assert(source.trim().isNotEmpty, 'Measured needs a non-empty source');
    return Measured._(value: value, source: source, measuredAt: measuredAt);
  }

  /// The measurement could not be taken.
  factory Measured.unknown({
    required String source,
    required DateTime measuredAt,
  }) {
    assert(source.trim().isNotEmpty, 'Measured needs a non-empty source');
    return Measured._(value: null, source: source, measuredAt: measuredAt);
  }

  /// Null means unknown.
  final T? value;
  final String source;
  final DateTime measuredAt;

  bool get isKnown => value != null;
}
