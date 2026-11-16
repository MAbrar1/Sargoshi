import 'package:flutter/foundation.dart'
    show LicenseEntryWithLineBreaks, LicenseRegistry;
import 'package:flutter/services.dart' show rootBundle;

const _fontLicenses = <(String, String)>[
  ('Inter', 'assets/fonts/licenses/inter-OFL.txt'),
  ('Instrument Serif', 'assets/fonts/licenses/instrumentserif-OFL.txt'),
  ('JetBrains Mono', 'assets/fonts/licenses/jetbrainsmono-OFL.txt'),
];

/// Makes the bundled fonts' licences appear in the app's licence page.
void registerFontLicenses() {
  LicenseRegistry.addLicense(() async* {
    for (final (name, path) in _fontLicenses) {
      yield LicenseEntryWithLineBreaks([
        name,
      ], await rootBundle.loadString(path));
    }
  });
}
