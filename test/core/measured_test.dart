import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/core/measured/measured.dart';

void main() {
  final t = DateTime.utc(2026, 11, 15);

  test('known value carries source and time', () {
    final m = Measured<int>.of(3, source: 'unit-test', measuredAt: t);
    expect(m.isKnown, isTrue);
    expect(m.value, 3);
    expect(m.source, 'unit-test');
    expect(m.measuredAt, t);
  });

  test('unknown value is never known', () {
    final m = Measured<bool>.unknown(source: 'unit-test', measuredAt: t);
    expect(m.isKnown, isFalse);
    expect(m.value, isNull);
  });

  test('empty source is rejected', () {
    expect(
      () => Measured<int>.of(1, source: ' ', measuredAt: t),
      throwsA(isA<AssertionError>()),
    );
  });
}
