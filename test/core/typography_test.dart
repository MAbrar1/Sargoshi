import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/core/design/typography.dart';

void main() {
  final all = {
    'displayXl': AppText.displayXl,
    'displayL': AppText.displayL,
    'titleL': AppText.titleL,
    'titleM': AppText.titleM,
    'titleS': AppText.titleS,
    'bodyL': AppText.bodyL,
    'chatBody': AppText.chatBody,
    'bodyM': AppText.bodyM,
    'labelL': AppText.labelL,
    'labelM': AppText.labelM,
    'safetyNumber': AppText.safetyNumber,
  };

  test('no style is below the 12 sp minimum', () {
    for (final e in all.entries) {
      expect(e.value.fontSize, greaterThanOrEqualTo(12), reason: e.key);
    }
  });

  test('line heights match the spec scale', () {
    double lh(String k) => all[k]!.fontSize! * all[k]!.height!;
    expect(lh('displayXl'), closeTo(48, 0.01));
    expect(lh('displayL'), closeTo(40, 0.01));
    expect(lh('titleL'), closeTo(30, 0.01));
    expect(lh('titleM'), closeTo(26, 0.01));
    expect(lh('titleS'), closeTo(24, 0.01));
    expect(lh('bodyL'), closeTo(24, 0.01));
    expect(lh('chatBody'), closeTo(22, 0.01));
    expect(lh('bodyM'), closeTo(22, 0.01));
    expect(lh('labelL'), closeTo(20, 0.01));
    expect(lh('labelM'), closeTo(16, 0.01));
    expect(lh('safetyNumber'), closeTo(22, 0.01));
  });

  test(
    'families: serif for display, mono for safety numbers, Inter otherwise',
    () {
      expect(AppText.displayXl.fontFamily, FontFamilies.display);
      expect(AppText.displayL.fontFamily, FontFamilies.display);
      expect(AppText.safetyNumber.fontFamily, FontFamilies.mono);
      for (final k in [
        'titleL',
        'titleM',
        'titleS',
        'bodyL',
        'bodyM',
        'labelL',
        'labelM',
      ]) {
        expect(all[k]!.fontFamily, FontFamilies.ui, reason: k);
      }
    },
  );

  test('label M has 2 percent tracking', () {
    expect(AppText.labelM.letterSpacing, closeTo(0.24, 0.001));
  });

  test('tabular figures are switched on by AppText.tabular', () {
    final s = AppText.tabular(AppText.bodyM);
    expect(s.fontFeatures, isNotNull);
    expect(s.fontFeatures!.map((f) => f.feature), contains('tnum'));
    expect(AppText.bodyM.fontFeatures, isNull);
  });
}
