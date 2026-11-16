import 'dart:ui' show FontFeature, FontVariation;

import 'package:flutter/painting.dart';

import 'tokens.dart';

/// Bundled font families (declared in pubspec.yaml). Never fetched at runtime.
abstract final class FontFamilies {
  static const ui = 'Inter';
  static const display = 'InstrumentSerif';
  static const mono = 'JetBrainsMono';
}

TextStyle _ui(
  ({double size, double height}) scale,
  int weight, {
  double tracking = 0,
}) => TextStyle(
  fontFamily: FontFamilies.ui,
  fontSize: scale.size,
  height: scale.height / scale.size,
  fontWeight: FontWeight.values[weight ~/ 100 - 1],
  fontVariations: [FontVariation('wght', weight.toDouble())],
  letterSpacing: tracking * scale.size,
);

TextStyle _display(({double size, double height}) scale) => TextStyle(
  fontFamily: FontFamilies.display,
  fontSize: scale.size,
  height: scale.height / scale.size,
);

/// Type scale from the spec (section 5.2). Colours are applied by the theme.
abstract final class AppText {
  static final displayXl = _display(TypeScale.displayXl);
  static final displayL = _display(TypeScale.displayL);
  static final titleL = _ui(TypeScale.titleL, 600);
  static final titleM = _ui(TypeScale.titleM, 600);
  static final titleS = _ui(TypeScale.titleS, 600);
  static final bodyL = _ui(TypeScale.bodyL, 400);

  /// Chat message text: Body L size with a 22 line height.
  static final chatBody = bodyL.copyWith(height: 22 / TypeScale.bodyL.size);
  static final bodyM = _ui(TypeScale.bodyM, 400);
  static final labelL = _ui(TypeScale.labelL, 500);

  /// Label M carries +2% tracking.
  static final labelM = _ui(TypeScale.labelM, 500, tracking: 0.02);

  /// Safety numbers and fingerprints: mono 15/22.
  static final safetyNumber = TextStyle(
    fontFamily: FontFamilies.mono,
    fontSize: 15,
    height: 22 / 15,
  );

  /// Times, counts, call timers and unread badges use tabular figures.
  static TextStyle tabular(TextStyle base) =>
      base.copyWith(fontFeatures: const [FontFeature.tabularFigures()]);
}
