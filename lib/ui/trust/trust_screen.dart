import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../application/trust/trust_controller.dart';
import '../../domain/trust/trust_state.dart';
import '../../l10n/app_localizations.dart';

final class TrustScreen extends ConsumerWidget {
  const TrustScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final trust = ref.watch(trustProvider);
    final trustValue = trust.measurement.value;
    final statusText = switch (trustValue) {
      null => l10n.trustStatusUnknown,
      TrustValue.unverified => l10n.trustStatusUnverified,
      TrustValue.verified => l10n.trustStatusVerified,
      TrustValue.keyChanged => l10n.trustStatusKeyChanged,
    };
    final gateText = switch (trust.gateState) {
      KeyChangeGateState.idle => l10n.trustIdle,
      KeyChangeGateState.awaitingReview => l10n.trustAwaitingReview,
      KeyChangeGateState.accepted => l10n.trustAccepted,
      KeyChangeGateState.blocked => l10n.trustBlocked,
    };

    return Scaffold(
      appBar: AppBar(title: Text(l10n.trustTitle)),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsetsDirectional.all(20),
          children: [
            Text(statusText, style: Theme.of(context).textTheme.headlineSmall),
            const SizedBox(height: 12),
            Text(l10n.trustDemoDisclosure),
            const SizedBox(height: 8),
            Text(
              trustValue == null
                  ? l10n.trustMeasurementUnavailable
                  : '${l10n.trustSource}: ${trust.measurement.source}',
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            if (trustValue != null) ...[
              const SizedBox(height: 8),
              Text(
                '${l10n.trustMeasuredAt}: ${trust.measurement.measuredAt.toUtc().toIso8601String()}',
                style: Theme.of(context).textTheme.bodyMedium,
              ),
            ],
            const SizedBox(height: 16),
            Card(
              child: Padding(
                padding: const EdgeInsetsDirectional.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      l10n.trustKeyChangeGate,
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(gateText),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            OutlinedButton(
              onPressed: trust.isAwaitingReview
                  ? null
                  : () => ref.read(trustProvider.notifier).simulateKeyChange(),
              child: Text(l10n.trustSimulateKeyChange),
            ),
            if (trust.isAwaitingReview) ...[
              const SizedBox(height: 8),
              FilledButton.tonal(
                onPressed: () =>
                    ref.read(trustProvider.notifier).acceptKeyChange(),
                child: Text(l10n.trustAcceptKeyChange),
              ),
              const SizedBox(height: 8),
              FilledButton(
                onPressed: () =>
                    ref.read(trustProvider.notifier).blockKeyChange(),
                child: Text(l10n.trustBlockKeyChange),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
