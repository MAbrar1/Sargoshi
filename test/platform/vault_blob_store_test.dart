import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/platform/vault_blob_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel(ChannelVaultBlobStore.channelName);
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  test('read, write, delete and list pass names, bytes and prefix', () async {
    final calls = <String>[];
    Object? sent;
    messenger.setMockMethodCallHandler(channel, (call) async {
      final args = call.arguments as Map<Object?, Object?>;
      calls.add('${call.method}:${args['name'] ?? args['prefix']}');
      if (call.method == 'write') sent = args['bytes'];
      return switch (call.method) {
        'read' => Uint8List.fromList([1, 2]),
        'list' => <Object?>['real/item/note/a1', 'real/item/note/b2'],
        _ => null,
      };
    });
    final store = ChannelVaultBlobStore();
    expect(await store.read('real/master'), Uint8List.fromList([1, 2]));
    await store.write('real/item/note/a1', Uint8List.fromList([3]));
    await store.delete('real/item/note/a1');
    expect(await store.list('real/item/note/'), [
      'real/item/note/a1',
      'real/item/note/b2',
    ]);
    expect(calls, [
      'read:real/master',
      'write:real/item/note/a1',
      'delete:real/item/note/a1',
      'list:real/item/note/',
    ]);
    expect(sent, Uint8List.fromList([3]));
  });

  test('a null list result is an empty list', () async {
    messenger.setMockMethodCallHandler(channel, (call) async => null);
    expect(await ChannelVaultBlobStore().list('real/'), isEmpty);
  });

  test('platform errors become VaultBlobsUnavailableException', () async {
    messenger.setMockMethodCallHandler(channel, (call) async {
      throw PlatformException(code: 'blobs_unavailable');
    });
    await expectLater(
      ChannelVaultBlobStore().read('real/master'),
      throwsA(isA<VaultBlobsUnavailableException>()),
    );
  });
}
