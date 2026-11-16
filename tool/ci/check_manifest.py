#!/usr/bin/env python3
"""Check the MERGED release manifest against the spec.

usage: check_manifest.py <merged AndroidManifest.xml>
Fails on: any permission not on the allow-list, allowBackup not false,
missing backup rule files, cleartext allowed, or any exported component
other than MainActivity (plus Flutter-free launcher). Prints the manifest.
"""
import sys
import xml.etree.ElementTree as ET

A = "{http://schemas.android.com/apk/res/android}"
ALLOWED = {
    "android.permission.INTERNET",
    # Added by the Android Gradle plugin or Flutter itself; reviewed:
    "app.sargoshi.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION",
}
ALLOWED_EXPORTED = {"app.sargoshi.MainActivity", ".MainActivity"}


def main(path: str) -> int:
    text = open(path).read()
    print(text)
    root = ET.fromstring(text)
    bad = []
    for tag in ("uses-permission", "uses-permission-sdk-23", "permission"):
        for e in root.iter(tag):
            n = e.get(A + "name")
            if n not in ALLOWED:
                bad.append(f"permission not on allow-list: {n}")
    app = root.find("application")
    if app.get(A + "allowBackup") != "false":
        bad.append("allowBackup must be false")
    if app.get(A + "usesCleartextTraffic") != "false":
        bad.append("usesCleartextTraffic must be false")
    for attr in ("fullBackupContent", "dataExtractionRules", "networkSecurityConfig"):
        if not app.get(A + attr):
            bad.append(f"missing application attribute {attr}")
    for kind in ("activity", "activity-alias", "service", "receiver", "provider"):
        for e in app.iter(kind):
            name = e.get(A + "name")
            exported = e.get(A + "exported")
            has_filter = e.find("intent-filter") is not None
            is_exp = exported == "true" or (exported is None and has_filter)
            if is_exp and name not in ALLOWED_EXPORTED:
                bad.append(f"exported component not allowed: {kind} {name}")
    if bad:
        print("\nMANIFEST CHECK FAILED:\n  " + "\n  ".join(bad))
        return 1
    print("\ncheck_manifest: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
