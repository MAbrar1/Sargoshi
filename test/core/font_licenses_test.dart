import 'package:flutter/foundation.dart' show LicenseRegistry;
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/core/licenses/font_licenses.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('font licences appear in the licence registry', () async {
    registerFontLicenses();
    final packages = <String>{};
    await for (final entry in LicenseRegistry.licenses) {
      packages.addAll(entry.packages);
    }
    expect(packages, containsAll(['Inter', 'Instrument Serif', 'JetBrains Mono']));
  });
}
