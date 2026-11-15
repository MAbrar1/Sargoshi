import 'package:material_ui/material_ui.dart';

import 'tokens.dart';

/// Font families are declared but not yet bundled (see STAGE-0B-report.md).
/// Until the font files are added, Flutter falls back to the system font.
const _ui = 'Inter';

TextTheme _textTheme(Color primary, Color secondary) => TextTheme(
      titleLarge: TextStyle(
          fontFamily: _ui,
          fontSize: TypeScale.titleL.size,
          height: TypeScale.titleL.height / TypeScale.titleL.size,
          fontWeight: FontWeight.w600,
          color: primary),
      titleMedium: TextStyle(
          fontFamily: _ui,
          fontSize: TypeScale.titleM.size,
          height: TypeScale.titleM.height / TypeScale.titleM.size,
          fontWeight: FontWeight.w600,
          color: primary),
      titleSmall: TextStyle(
          fontFamily: _ui,
          fontSize: TypeScale.titleS.size,
          height: TypeScale.titleS.height / TypeScale.titleS.size,
          fontWeight: FontWeight.w600,
          color: primary),
      bodyLarge: TextStyle(
          fontFamily: _ui,
          fontSize: TypeScale.bodyL.size,
          height: TypeScale.bodyL.height / TypeScale.bodyL.size,
          color: primary),
      bodyMedium: TextStyle(
          fontFamily: _ui,
          fontSize: TypeScale.bodyM.size,
          height: TypeScale.bodyM.height / TypeScale.bodyM.size,
          color: secondary),
      labelLarge: TextStyle(
          fontFamily: _ui,
          fontSize: TypeScale.labelL.size,
          height: TypeScale.labelL.height / TypeScale.labelL.size,
          fontWeight: FontWeight.w500,
          color: primary),
      labelMedium: TextStyle(
          fontFamily: _ui,
          fontSize: TypeScale.labelM.size,
          height: TypeScale.labelM.height / TypeScale.labelM.size,
          fontWeight: FontWeight.w500,
          letterSpacing: 0.24,
          color: secondary),
    );

ThemeData buildDarkTheme() => ThemeData(
      brightness: Brightness.dark,
      useMaterial3: true,
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
