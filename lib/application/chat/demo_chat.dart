import 'package:flutter_riverpod/flutter_riverpod.dart';

final demoChatsProvider = Provider<List<DemoChat>>((ref) => const [
  DemoChat(id: 'maya', nameKey: 'demoMayaName', previewKey: 'demoMayaPreview', timeKey: 'demoTimeNow', unread: 2),
  DemoChat(id: 'omar', nameKey: 'demoOmarName', previewKey: 'demoOmarPreview', timeKey: 'demoTimeEarlier', unread: 0),
  DemoChat(id: 'saved', nameKey: 'savedMessages', previewKey: 'demoSavedPreview', timeKey: 'demoTimeYesterday', unread: 0),
]);

final class DemoChat {
  const DemoChat({
    required this.id,
    required this.nameKey,
    required this.previewKey,
    required this.timeKey,
    required this.unread,
  });

  final String id;
  final String nameKey;
  final String previewKey;
  final String timeKey;
  final int unread;
}

/// Ephemeral in-memory messages for Stage 1 UI work. This is not a network or
/// persistence layer, and sending here does not deliver a message.
final class DemoMessage {
  const DemoMessage({required this.text, required this.isOutgoing});
  final String text;
  final bool isOutgoing;
}
