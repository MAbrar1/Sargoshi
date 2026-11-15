# ADR-001: Flutter and Dart version

Status: ACCEPTED (measured).
Decision: pin Flutter 3.50.0 (stable channel as installed by CI on 2026-11-15).
Evidence: the CI log shows FLUTTER_ROOT stable-3.50.0. An earlier web search suggested 3.47.1, which was out of date.
Consequences: Material and Cupertino live in the material_ui and cupertino_ui packages since 3.47; import those, not package:flutter/material.dart.
TODO(verify): Dart SDK version bundled with 3.50.0 (print `flutter --version` in CI); pin material_ui and cupertino_ui versions by committing pubspec.lock.
