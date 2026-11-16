import 'package:cupertino_ui/cupertino_ui.dart';
import 'package:flutter_localizations/flutter_localizations.dart'
    show GlobalWidgetsLocalizations;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'core/design/theme.dart';
import 'core/licenses/font_licenses.dart';
import 'l10n/app_localizations.dart';
import 'ui/chat/chat_home_screen.dart';
import 'ui/shared/demo_crypto_banner.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  registerFontLicenses();
  runApp(const ProviderScope(child: SargoshiApp()));
}

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
    // The demo banner sits above every route, so no screen can hide it.
    builder: (context, child) => Column(
      children: [
        const DemoCryptoBanner(),
        Expanded(
          child: MediaQuery.removePadding(
            context: context,
            removeTop: true,
            child: child ?? const SizedBox.shrink(),
          ),
        ),
      ],
    ),
    home: const ChatHomeScreen(),
  );
}
