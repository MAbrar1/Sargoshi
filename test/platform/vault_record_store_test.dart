import 'dart:typed_data';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sargoshi/domain/vault/vault_records.dart';
import 'package:sargoshi/platform/vault_record_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel(ChannelVaultRecordStore.channelName);
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  test('read, write and delete pass the slot name and bytes', () async {
    final calls = <String>[];
    Object? sent;
    messenger.setMockMethodCallHandler(channel, (call) async {
      final args = call.arguments as Map<Object?, Object?>;
      calls.add('${call.method}:${args['name']}');
      if (call.method == 'write') sent = args['bytes'];
      return call.method == 'read' ? Uint8List.fromList([1, 2]) : null;
    });
    final store = ChannelVaultRecordStore();
    expect(await store.read(VaultSlot.real), Uint8List.fromList([1, 2]));
    await store.write(VaultSlot.decoy, Uint8List.fromList([3]));
    await store.delete(VaultSlot.decoy);
    expect(calls, ['read:real', 'write:decoy', 'delete:decoy']);
    expect(sent, Uint8List.fromList([3]));
  });

  test('platform errors become VaultRecordsUnavailableException', () async {
    messenger.setMockMethodCallHandler(channel, (call) async {
      throw PlatformException(code: 'records_unavailable');
    });
    await expectLater(
      ChannelVaultRecordStore().read(VaultSlot.real),
      throwsA(isA<VaultRecordsUnavailableException>()),
    );
  });
}
