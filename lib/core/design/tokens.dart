import 'package:flutter/painting.dart';

/// Moonlit Ember colour tokens. Single source of truth: the CI script
/// tool/ci/check_contrast.py parses the `static const name = Color(0xFF......)`
/// lines below, so keep that exact shape.
abstract final class DarkTokens {
  static const ink = Color(0xFF0B0D1A);
  static const dusk = Color(0xFF161A33);
  static const dusk2 = Color(0xFF20254A);
  static const line = Color(0xFF2B3160);
  static const moon = Color(0xFFEDEBFF);
  static const mist = Color(0xFFA7A9C7);
  static const glow = Color(0xFFFF8A4C);
  static const mint = Color(0xFF4ADE9C);
  static const amber = Color(0xFFFFC857);
  static const coral = Color(0xFFFF6B6B);
  static const emberStart = Color(0xFFC2410C);
  static const roseEnd = Color(0xFFBE185D);
  static const white = Color(0xFFFFFFFF);
}

abstract final class LightTokens {
  static const parchment = Color(0xFFFAF7F2);
  static const paper = Color(0xFFFFFFFF);
  static const sand = Color(0xFFF0EBE2);
  static const line = Color(0xFFE3DCCF);
  static const text = Color(0xFF17142A);
  static const secondary = Color(0xFF5B5E7A);
  static const emberDeep = Color(0xFFB8400A);
  static const mintDeep = Color(0xFF0F7A4F);
  static const amberDeep = Color(0xFF8A5A00);
  static const coralDeep = Color(0xFFC23B3B);
}

/// Outgoing bubble and primary button gradient, 135 degrees, white text.
const emberRoseGradient = LinearGradient(
  begin: AlignmentDirectional.topStart,
  end: AlignmentDirectional.bottomEnd,
  colors: [DarkTokens.emberStart, DarkTokens.roseEnd],
);

/// Radii from the spec (dp).
abstract final class Radii {
  static const bubble = 18.0;
  static const bubbleJoined = 6.0;
  static const card = 14.0;
  static const input = 14.0;
  static const chip = 999.0;
  static const sheetTop = 24.0;
}

/// Type scale from the spec: size and line height in sp.
abstract final class TypeScale {
  static const displayXl = (size: 44.0, height: 48.0);
  static const displayL = (size: 34.0, height: 40.0);
  static const titleL = (size: 24.0, height: 30.0);
  static const titleM = (size: 20.0, height: 26.0);
  static const titleS = (size: 17.0, height: 24.0);
  static const bodyL = (size: 16.0, height: 24.0);
  static const bodyM = (size: 15.0, height: 22.0);
  static const labelL = (size: 14.0, height: 20.0);
  static const labelM = (size: 12.0, height: 16.0);
}
