import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/core/design/typography.dart';

/// Spec section 5.2: bundled, subsetted, OFL, total payload under 250 KB.
void main() {
  const fontFiles = [
    'assets/fonts/Inter-Variable.ttf',
    'assets/fonts/InstrumentSerif-Regular.ttf',
    'assets/fonts/InstrumentSerif-Italic.ttf',
    'assets/fonts/JetBrainsMono-Regular.ttf',
  ];
  const licenceFiles = [
    'assets/fonts/licenses/inter-OFL.txt',
    'assets/fonts/licenses/instrumentserif-OFL.txt',
    'assets/fonts/licenses/jetbrainsmono-OFL.txt',
  ];

  test('every font file and licence exists', () {
    for (final p in [...fontFiles, ...licenceFiles]) {
      expect(File(p).existsSync(), isTrue, reason: p);
    }
  });

  test('total font payload is under 250 KB', () {
    final total = fontFiles.fold<int>(0, (a, p) => a + File(p).lengthSync());
    expect(total, lessThan(250 * 1000));
  });

  test('licences are the SIL Open Font License', () {
    for (final p in licenceFiles) {
      expect(
        File(p).readAsStringSync().toUpperCase(),
        contains('SIL OPEN FONT LICENSE'),
        reason: p,
      );
    }
  });

  test('pubspec declares every family used in code', () {
    final pubspec = File('pubspec.yaml').readAsStringSync();
    for (final family in [
      FontFamilies.ui,
      FontFamilies.display,
      FontFamilies.mono,
    ]) {
      expect(pubspec, contains('family: $family'));
    }
    for (final p in fontFiles) {
      expect(pubspec, contains(p));
    }
  });

  test('no runtime font fetching dependency', () {
    expect(
      File('pubspec.yaml').readAsStringSync(),
      isNot(contains('google_fonts')),
    );
  });
}
