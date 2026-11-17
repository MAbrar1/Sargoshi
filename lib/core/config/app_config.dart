/// Build-time configuration. Values come from `--dart-define` so no real
/// domain or key is committed. See ADR-006 and `tool/ci/check_release_ready.py`.
abstract final class AppConfig {
  /// Placeholder until the owner supplies the real domain.
  static const placeholderDomain = 'sargoshi.example';

  /// Host used for App Links and the pinned API endpoint.
  static const domain = String.fromEnvironment(
    'SARGOSHI_DOMAIN',
    defaultValue: placeholderDomain,
  );

  static bool get isPlaceholderDomain => domain == placeholderDomain;

  /// True while only the fake crypto core exists. A release build must be
  /// made with `--dart-define=SARGOSHI_FAKE_CRYPTO=false`, which then fails
  /// at startup until a production core is registered.
  static const useFakeCrypto = bool.fromEnvironment(
    'SARGOSHI_FAKE_CRYPTO',
    defaultValue: true,
  );
}
