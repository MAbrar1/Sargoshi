import 'package:material_ui/material_ui.dart';

import 'tokens.dart';
import 'typography.dart';


TextTheme _textTheme(Color primary, Color secondary) => TextTheme(
  displayLarge: AppText.displayXl.copyWith(color: primary),
  displayMedium: AppText.displayL.copyWith(color: primary),
  titleLarge: AppText.titleL.copyWith(color: primary),
  titleMedium: AppText.titleM.copyWith(color: primary),
  titleSmall: AppText.titleS.copyWith(color: primary),
  bodyLarge: AppText.bodyL.copyWith(color: primary),
  bodyMedium: AppText.bodyM.copyWith(color: secondary),
  labelLarge: AppText.labelL.copyWith(color: primary),
  labelMedium: AppText.labelM.copyWith(color: secondary),
);

ThemeData buildDarkTheme() => ThemeData(
  brightness: Brightness.dark,
  useMaterial3: true,
  fontFamily: FontFamilies.ui,
  scaffoldBackgroundColor: DarkTokens.ink,
  colorScheme: const ColorScheme.dark(
    surface: DarkTokens.dusk,
    primary: DarkTokens.glow,
    onSurface: DarkTokens.moon,
    error: DarkTokens.coral,
  ),
  dividerColor: DarkTokens.line,
  textTheme: _textTheme(DarkTokens.moon, DarkTokens.mist),
);

ThemeData buildLightTheme() => ThemeData(
  brightness: Brightness.light,
  useMaterial3: true,
  fontFamily: FontFamilies.ui,
  scaffoldBackgroundColor: LightTokens.parchment,
  colorScheme: const ColorScheme.light(
    surface: LightTokens.paper,
    primary: LightTokens.emberDeep,
    onSurface: LightTokens.text,
    error: LightTokens.coralDeep,
  ),
  dividerColor: LightTokens.line,
  textTheme: _textTheme(LightTokens.text, LightTokens.secondary),
);
