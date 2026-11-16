import 'package:cupertino_ui/cupertino_ui.dart';
import 'package:flutter_localizations/flutter_localizations.dart'
    show GlobalWidgetsLocalizations;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'core/design/theme.dart';
import 'l10n/app_localizations.dart';

void main() => runApp(const ProviderScope(child: SargoshiApp()));

class SargoshiApp extends StatelessWidget {
  const SargoshiApp({super.key});

  @override
  Widget build(BuildContext context) => MaterialApp(
    onGenerateTitle: (c) => AppLocalizations.of(c).appName,
    theme: buildLightTheme(),
    darkTheme: buildDarkTheme(),
    themeMode: ThemeMode.dark,
    localizationsDelegates: const [
      AppLocalizations.delegate,
      GlobalMaterialLocalizations.delegate,
      GlobalWidgetsLocalizations.delegate,
      GlobalCupertinoLocalizations.delegate,
    ],
    supportedLocales: AppLocalizations.supportedLocales,
    home: const _Placeholder(),
  );
}

/// Stage 0B placeholder. Real screens start in Stage 1.
class _Placeholder extends StatelessWidget {
  const _Placeholder();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(body: Center(child: Text(l10n.appName)));
  }
}
