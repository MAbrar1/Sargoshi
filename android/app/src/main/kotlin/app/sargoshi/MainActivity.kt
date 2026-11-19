package app.sargoshi

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import android.os.SystemClock
import android.provider.Settings
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
        registerClock(flutterEngine)
        val records = VaultRecordFiles(applicationContext)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, RECORDS_CHANNEL)
            .setMethodCallHandler { call, result ->
                try {
                    val name = call.argument<String>("name") ?: throw IllegalArgumentException()
                    when (call.method) {
                        "read" -> result.success(records.read(name))
                        "write" -> {
                            val bytes = call.argument<ByteArray>("bytes") ?: throw IllegalArgumentException()
                            records.write(name, bytes)
                            result.success(null)
                        }
                        "delete" -> {
                            records.delete(name)
                            result.success(null)
                        }
                        else -> result.notImplemented()
                    }
                } catch (e: Exception) {
                    result.error("records_unavailable", null, null)
                }
            }
        val blobs = VaultBlobFiles(applicationContext)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, BLOBS_CHANNEL)
            .setMethodCallHandler { call, result ->
                try {
                    when (call.method) {
                        "list" -> {
                            val prefix = call.argument<String>("prefix") ?: throw IllegalArgumentException()
                            result.success(blobs.list(prefix))
                        }
                        else -> {
                            val name = call.argument<String>("name") ?: throw IllegalArgumentException()
                            when (call.method) {
                                "read" -> result.success(blobs.read(name))
                                "write" -> {
                                    val bytes = call.argument<ByteArray>("bytes") ?: throw IllegalArgumentException()
                                    blobs.write(name, bytes)
                                    result.success(null)
                                }
                                "delete" -> {
                                    blobs.delete(name)
                                    result.success(null)
                                }
                                else -> result.notImplemented()
                            }
                        }
                    }
                } catch (e: Exception) {
                    result.error("blobs_unavailable", null, null)
                }
            }
    }

    private fun registerClock(flutterEngine: FlutterEngine) {
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CLOCK_CHANNEL)
            .setMethodCallHandler { call, result ->
                if (call.method == "now") {
                    // Monotonic time ignores edits to the date and time. The boot
                    // count changes on every restart; -1 means it could not be read.
                    val boot = Settings.Global.getInt(contentResolver, Settings.Global.BOOT_COUNT, -1)
                    result.success(mapOf("elapsedMs" to SystemClock.elapsedRealtime(), "boot" to boot))
                } else {
                    result.notImplemented()
                }
            }
    }

    private companion object {
        const val CLOCK_CHANNEL = "app.sargoshi/clock"
        const val PEPPER_CHANNEL = "app.sargoshi/vault_pepper"
        const val RECORDS_CHANNEL = "app.sargoshi/vault_records"
        const val BLOBS_CHANNEL = "app.sargoshi/vault_blobs"
    }
}
