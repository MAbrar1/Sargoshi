import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'demo_chat.dart';

final demoMessagesProvider = StateProvider.family<List<DemoMessage>, String>(
  (ref, chatId) => switch (chatId) {
    'maya' => const [
      DemoMessage(text: 'demoMayaMessage1', isOutgoing: false),
      DemoMessage(text: 'demoMayaMessage2', isOutgoing: true),
      DemoMessage(text: 'demoMayaMessage3', isOutgoing: false),
    ],
    'omar' => const [
      DemoMessage(text: 'demoOmarMessage1', isOutgoing: false),
      DemoMessage(text: 'demoOmarMessage2', isOutgoing: true),
    ],
    _ => const [],
  },
);
