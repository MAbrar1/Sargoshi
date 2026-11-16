import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../application/chat/demo_chat.dart';
import '../../application/chat/demo_messages.dart';
import '../../application/trust/trust_controller.dart';
import '../../core/design/tokens.dart';
import '../../domain/trust/trust_state.dart';
import '../../l10n/app_localizations.dart';
import '../vault/vault_screen.dart';

final class ChatHomeScreen extends ConsumerStatefulWidget {
  const ChatHomeScreen({super.key});

  @override
  ConsumerState<ChatHomeScreen> createState() => _ChatHomeScreenState();
}

class _ChatHomeScreenState extends ConsumerState<ChatHomeScreen> {
  String? _selectedChatId;
  String _query = '';

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final chats = ref.watch(demoChatsProvider);
    final matches = chats
        .where((chat) {
          final name = _chatName(l10n, chat);
          return name.toLowerCase().contains(_query.toLowerCase());
        })
        .toList(growable: false);
    final width = MediaQuery.sizeOf(context).width;
    final wide = width >= 600;
    final selected = _selectedChatId == null
        ? null
        : chats.where((chat) => chat.id == _selectedChatId).firstOrNull;

    return Scaffold(
      body: Column(
        children: [
          Expanded(
            child: wide
                ? Row(
                    children: [
                      SizedBox(
                        width: width >= 840 ? 380 : 320,
                        child: _buildChatList(context, l10n, matches),
                      ),
                      const VerticalDivider(width: 1),
                      Expanded(
                        child: selected == null
                            ? _EmptySelection(l10n: l10n)
                            : _DemoThread(chat: selected),
                      ),
                    ],
                  )
                : selected == null
                ? _buildChatList(context, l10n, matches)
                : _DemoThread(
                    chat: selected,
                    onBack: () => setState(() => _selectedChatId = null),
                  ),
          ),
        ],
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: 0,
        destinations: [
          NavigationDestination(
            icon: const Icon(Icons.chat_bubble_outline),
            label: l10n.tabChats,
          ),
          NavigationDestination(
            icon: const Icon(Icons.call_outlined),
            label: l10n.tabCalls,
          ),
          NavigationDestination(
            icon: const Icon(Icons.shield_outlined),
            label: l10n.tabGuard,
          ),
          NavigationDestination(
            icon: const Icon(Icons.person_outline),
            label: l10n.tabYou,
          ),
        ],
      ),
    );
  }

  Widget _buildChatList(
    BuildContext context,
    AppLocalizations l10n,
    List<DemoChat> chats,
  ) => CustomScrollView(
    slivers: [
      SliverPadding(
        padding: const EdgeInsetsDirectional.fromSTEB(20, 20, 20, 8),
        sliver: SliverToBoxAdapter(
          // Hidden entry (spec 6.1): long-press the wordmark to open the vault.
          child: GestureDetector(
            onLongPress: () => Navigator.of(context).push(
              MaterialPageRoute<void>(builder: (_) => const VaultScreen()),
            ),
            child: Text(
              l10n.appName,
              style: Theme.of(context).textTheme.displayMedium,
            ),
          ),
        ),
      ),
      SliverPadding(
        padding: const EdgeInsetsDirectional.fromSTEB(16, 0, 16, 12),
        sliver: SliverToBoxAdapter(
          child: SearchBar(
            hintText: l10n.searchChats,
            leading: const Icon(Icons.search),
            onChanged: (value) => setState(() => _query = value.trim()),
          ),
        ),
      ),
      SliverPadding(
        padding: const EdgeInsetsDirectional.symmetric(horizontal: 16),
        sliver: SliverToBoxAdapter(
          child: Card(
            child: ListTile(
              leading: const Icon(Icons.mark_email_unread_outlined),
              title: Text(l10n.messageRequests),
              subtitle: Text(l10n.messageRequestsDescription),
              trailing: const Icon(Icons.chevron_right),
            ),
          ),
        ),
      ),
      if (chats.isEmpty)
        SliverFillRemaining(
          hasScrollBody: false,
          child: Center(child: Text(l10n.noChatsFound)),
        )
      else
        SliverList.builder(
          itemCount: chats.length,
          itemBuilder: (context, index) {
            final chat = chats[index];
            return ListTile(
              contentPadding: const EdgeInsetsDirectional.symmetric(
                horizontal: 20,
                vertical: 4,
              ),
              leading: CircleAvatar(
                backgroundColor: DarkTokens.dusk2,
                child: Text(_chatInitial(l10n, chat)),
              ),
              title: Text(_chatName(l10n, chat)),
              subtitle: Text(
                _chatPreview(l10n, chat),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              trailing: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text(
                    _chatTime(l10n, chat),
                    style: Theme.of(context).textTheme.labelSmall,
                  ),
                  if (chat.unread > 0)
                    Semantics(
                      label: l10n.unreadCount(chat.unread),
                      child: Container(
                        margin: const EdgeInsetsDirectional.only(top: 4),
                        padding: const EdgeInsetsDirectional.symmetric(
                          horizontal: 7,
                          vertical: 2,
                        ),
                        decoration: const BoxDecoration(
                          color: DarkTokens.emberStart,
                          borderRadius: BorderRadius.all(
                            Radius.circular(Radii.chip),
                          ),
                        ),
                        child: Text(l10n.unreadCount(chat.unread)),
                      ),
                    ),
                ],
              ),
              onTap: () => setState(() => _selectedChatId = chat.id),
            );
          },
        ),
    ],
  );
}

String _chatName(AppLocalizations l10n, DemoChat chat) =>
    switch (chat.nameKey) {
      'demoMayaName' => l10n.demoMayaName,
      'demoOmarName' => l10n.demoOmarName,
      _ => l10n.savedMessages,
    };

String _chatPreview(AppLocalizations l10n, DemoChat chat) =>
    switch (chat.previewKey) {
      'demoMayaPreview' => l10n.demoMayaPreview,
      'demoOmarPreview' => l10n.demoOmarPreview,
      _ => l10n.demoSavedPreview,
    };

String _chatTime(AppLocalizations l10n, DemoChat chat) =>
    switch (chat.timeKey) {
      'demoTimeNow' => l10n.demoTimeNow,
      'demoTimeEarlier' => l10n.demoTimeEarlier,
      _ => l10n.demoTimeYesterday,
    };

String _chatInitial(AppLocalizations l10n, DemoChat chat) => switch (chat.id) {
  'maya' => l10n.demoMayaInitial,
  'omar' => l10n.demoOmarInitial,
  _ => l10n.demoSavedInitial,
};

final class _EmptySelection extends StatelessWidget {
  const _EmptySelection({required this.l10n});
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) =>
      Center(child: Text(l10n.selectChat, textAlign: TextAlign.center));
}

final class _DemoThread extends ConsumerStatefulWidget {
  const _DemoThread({required this.chat, this.onBack});
  final DemoChat chat;
  final VoidCallback? onBack;

  @override
  ConsumerState<_DemoThread> createState() => _DemoThreadState();
}

class _DemoThreadState extends ConsumerState<_DemoThread> {
  final _composer = TextEditingController();

  @override
  void dispose() {
    _composer.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final messages = ref.watch(demoMessagesProvider(widget.chat.id));
    final trust = ref.watch(trustProvider(widget.chat.id));
    final pending = ref.watch(pendingDemoMessagesProvider(widget.chat.id));
    final isHeld = !trust.canSend;
    return Column(
      children: [
        AppBar(
          leading: widget.onBack == null
              ? null
              : IconButton(
                  tooltip: l10n.back,
                  onPressed: widget.onBack,
                  icon: const Icon(Icons.arrow_back),
                ),
          title: Text(_chatName(l10n, widget.chat)),
        ),
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(16, 4, 16, 12),
          child: Row(
            children: [
              const Icon(Icons.info_outline, size: 18),
              const SizedBox(width: 8),
              Expanded(child: Text(l10n.demoThreadDisclosure)),
            ],
          ),
        ),
        _DemoKeyChangeGate(
          chatId: widget.chat.id,
          state: trust.gateState,
          pendingCount: pending.length,
        ),
        Expanded(
          child: ListView.builder(
            padding: const EdgeInsetsDirectional.symmetric(horizontal: 16),
            itemCount: messages.length,
            itemBuilder: (context, index) {
              final message = messages[index];
              final text = _messageText(l10n, message.text);
              return Align(
                alignment: message.isOutgoing
                    ? AlignmentDirectional.centerEnd
                    : AlignmentDirectional.centerStart,
                child: Container(
                  constraints: const BoxConstraints(maxWidth: 560),
                  margin: const EdgeInsetsDirectional.only(bottom: 10),
                  padding: const EdgeInsetsDirectional.symmetric(
                    horizontal: 14,
                    vertical: 10,
                  ),
                  decoration: BoxDecoration(
                    color: message.isOutgoing
                        ? DarkTokens.emberStart
                        : Theme.of(context).colorScheme.surface,
                    borderRadius: BorderRadius.circular(Radii.bubble),
                  ),
                  child: Text(text),
                ),
              );
            },
          ),
        ),
        SafeArea(
          top: false,
          child: Padding(
            padding: const EdgeInsetsDirectional.fromSTEB(12, 8, 12, 12),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _composer,
                    enabled: !isHeld,
                    decoration: InputDecoration(
                      hintText: isHeld
                          ? l10n.demoSendHeld
                          : l10n.messageInputHint,
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(Radii.input),
                      ),
                    ),
                    textInputAction: TextInputAction.send,
                    onSubmitted: (_) => _send(l10n),
                  ),
                ),
                IconButton.filled(
                  tooltip: l10n.sendDemoMessage,
                  onPressed: isHeld ? null : () => _send(l10n),
                  icon: const Icon(Icons.send),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  void _send(AppLocalizations l10n) {
    final text = _composer.text.trim();
    if (text.isEmpty) return;
    final trust = ref.read(trustProvider(widget.chat.id));
    if (trust.canSend) {
      _appendDemoMessage(text);
    } else {
      final queue = ref.read(
        pendingDemoMessagesProvider(widget.chat.id).notifier,
      );
      queue.state = [...queue.state, text];
    }
    _composer.clear();
  }

  void _appendDemoMessage(String text) {
    final notifier = ref.read(demoMessagesProvider(widget.chat.id).notifier);
    notifier.state = [
      ...notifier.state,
      DemoMessage(text: text, isOutgoing: true),
    ];
  }
}

final class _DemoKeyChangeGate extends ConsumerWidget {
  const _DemoKeyChangeGate({
    required this.chatId,
    required this.state,
    required this.pendingCount,
  });

  final String chatId;
  final KeyChangeGateState state;
  final int pendingCount;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final controller = ref.read(trustProvider(chatId).notifier);
    final text = switch (state) {
      KeyChangeGateState.idle => l10n.demoTrustUnknown,
      KeyChangeGateState.awaitingReview => l10n.demoTrustAwaitingReview(
        pendingCount,
      ),
      KeyChangeGateState.accepted => l10n.demoTrustAccepted,
      KeyChangeGateState.blocked => l10n.demoTrustBlocked(pendingCount),
    };
    return Card(
      margin: const EdgeInsetsDirectional.fromSTEB(12, 0, 12, 8),
      child: Padding(
        padding: const EdgeInsetsDirectional.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.info_outline),
                const SizedBox(width: 8),
                Expanded(child: Text(text)),
                if (state == KeyChangeGateState.idle ||
                    state == KeyChangeGateState.accepted)
                  TextButton(
                    onPressed: controller.simulateKeyChange,
                    child: Text(l10n.demoSimulateKeyChange),
                  ),
              ],
            ),
            if (state == KeyChangeGateState.awaitingReview) ...[
              const SizedBox(height: 8),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  FilledButton.tonal(
                    onPressed: () {
                      controller.acceptKeyChange();
                      final queue = ref.read(
                        pendingDemoMessagesProvider(chatId).notifier,
                      );
                      final messages = ref.read(
                        demoMessagesProvider(chatId).notifier,
                      );
                      messages.state = [
                        ...messages.state,
                        ...queue.state.map(
                          (text) => DemoMessage(text: text, isOutgoing: true),
                        ),
                      ];
                      queue.state = const [];
                    },
                    child: Text(l10n.trustAcceptKeyChange),
                  ),
                  OutlinedButton(
                    onPressed: controller.blockKeyChange,
                    child: Text(l10n.trustBlockKeyChange),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }
}

String _messageText(AppLocalizations l10n, String key) => switch (key) {
  'demoMayaMessage1' => l10n.demoMayaMessage1,
  'demoMayaMessage2' => l10n.demoMayaMessage2,
  'demoMayaMessage3' => l10n.demoMayaMessage3,
  'demoOmarMessage1' => l10n.demoOmarMessage1,
  'demoOmarMessage2' => l10n.demoOmarMessage2,
  _ => key,
};
