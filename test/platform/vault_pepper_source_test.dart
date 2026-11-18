import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/platform/vault_pepper_source.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel(KeystorePepperSource.channelName);
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  test(
    'read, createIfMissing and exists use the right channel methods',
    () async {
      final calls = <String>[];
      messenger.setMockMethodCallHandler(channel, (call) async {
        calls.add(call.method);
        return switch (call.method) {
          'exists' => true,
          _ => Uint8List.fromList(List.filled(32, 9)),
        };
      });
      final source = KeystorePepperSource();
      expect((await source.pepper()).length, 32);
      expect((await source.createIfMissing()).length, 32);
      expect(await source.exists(), isTrue);
      expect(calls, ['read', 'createIfMissing', 'exists']);
    },
  );

  test('platform errors become PepperUnavailableException', () async {
    messenger.setMockMethodCallHandler(channel, (call) async {
      throw PlatformException(code: 'pepper_unavailable');
    });
    final source = KeystorePepperSource();
    await expectLater(
      source.pepper(),
      throwsA(isA<PepperUnavailableException>()),
    );
    await expectLater(
      source.exists(),
      throwsA(isA<PepperUnavailableException>()),
    );
  });

  test('a secret of the wrong length is rejected', () async {
    messenger.setMockMethodCallHandler(channel, (call) async => Uint8List(5));
    await expectLater(
      KeystorePepperSource().pepper(),
      throwsA(isA<PepperUnavailableException>()),
    );
  });
}
