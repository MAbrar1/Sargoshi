import 'dart:typed_data';

import 'package:flutter/services.dart' show MethodChannel, PlatformException;

import '../application/vault/attempt_limiter.dart';

/// Raised when the attempt counter or the clock cannot be reached. The vault
/// treats this as "cannot check", never as permission to try.
final class AttemptUnavailableException implements Exception {
  const AttemptUnavailableException();

  @override
  String toString() => 'AttemptUnavailableException';
}

/// Attempt counter kept in the vault record files (`attempts`).
final class ChannelAttemptStore implements AttemptStore {
  ChannelAttemptStore({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(channelName);

  static const channelName = 'app.sargoshi/vault_records';
  static const _name = 'attempts';

  final MethodChannel _channel;

  @override
  Future<Uint8List?> read() async {
    try {
      return await _channel.invokeMethod<Uint8List>('read', {'name': _name});
    } on PlatformException {
      throw const AttemptUnavailableException();
    }
  }

  @override
  Future<void> write(Uint8List bytes) async {
    try {
      await _channel.invokeMethod<void>('write', {
        'name': _name,
        'bytes': bytes,
      });
    } on PlatformException {
      throw const AttemptUnavailableException();
    }
  }
}

/// Monotonic clock and boot counter from Android.
final class ChannelAttemptClock implements AttemptClock {
  ChannelAttemptClock({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(channelName);

  static const channelName = 'app.sargoshi/clock';

  final MethodChannel _channel;

  @override
  Future<AttemptTime> now() async {
    try {
      final map = await _channel.invokeMapMethod<String, int>('now');
      final elapsed = map?['elapsedMs'];
      final boot = map?['boot'];
      if (elapsed == null || boot == null) {
        throw const AttemptUnavailableException();
      }
      return (elapsedMs: elapsed, boot: boot);
    } on PlatformException {
      throw const AttemptUnavailableException();
    }
  }
}
