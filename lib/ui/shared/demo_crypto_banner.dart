import 'package:material_ui/material_ui.dart';

import '../../core/design/tokens.dart';
import '../../l10n/app_localizations.dart';

/// Deliberately visible in every Stage 1 screen. This build has no crypto or
/// network implementation; the warning must remain until a real core exists.
final class DemoCryptoBanner extends StatelessWidget {
  const DemoCryptoBanner({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Material(
      color: DarkTokens.coral,
      child: SafeArea(
        bottom: false,
        child: SizedBox(
          width: double.infinity,
          child: Padding(
            padding: const EdgeInsetsDirectional.symmetric(
              horizontal: 16,
              vertical: 8,
            ),
            child: Text(
              l10n.demoCryptoBanner,
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: DarkTokens.ink,
                fontWeight: FontWeight.w800,
                letterSpacing: 0.6,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
