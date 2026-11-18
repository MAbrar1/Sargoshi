package app.sargoshi

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        PerformanceApi.setUp(
            flutterEngine.dartExecutor.binaryMessenger,
            PerformanceApiImpl(applicationContext),
        )
        val pepper = VaultPepperStore(applicationContext)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, PEPPER_CHANNEL)
            .setMethodCallHandler { call, result ->
                try {
                    when (call.method) {
                        "exists" -> result.success(pepper.exists())
                        "read" -> result.success(pepper.read())
                        "createIfMissing" -> result.success(pepper.createIfMissing())
                        else -> result.notImplemented()
                    }
                } catch (e: Exception) {
                    // Deliberately no message: nothing about the secret may leak.
                    result.error("pepper_unavailable", null, null)
                }
            }
    }

    private companion object {
        const val PEPPER_CHANNEL = "app.sargoshi/vault_pepper"
    }
}
