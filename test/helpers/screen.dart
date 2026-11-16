import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';

/// Sets a realistic logical screen. `setSurfaceSize` alone is in physical
/// pixels, and the default test pixel ratio of 3 would turn a "412 wide"
/// phone into a 137 dp one.
void useScreen(
  WidgetTester tester, {
  Size size = const Size(412, 915),
  double textScale = 1.0,
}) {
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1.0;
  tester.platformDispatcher.textScaleFactorTestValue = textScale;
  addTearDown(tester.view.reset);
  addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
}
