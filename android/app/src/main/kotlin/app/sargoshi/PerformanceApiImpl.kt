package app.sargoshi

import android.app.ActivityManager
import android.content.Context
import android.hardware.display.DisplayManager
import android.os.Build
import android.os.PowerManager
import android.view.Display

/**
 * Measures raw device facts. It never guesses: a value the platform cannot
 * provide is returned as null. Classification into tiers happens in Dart
 * (domain/performance/performance_tier.dart), where it is unit-tested.
 */
class PerformanceApiImpl(private val context: Context) : PerformanceApi {

    override fun getDeviceMetrics(): DeviceMetrics {
        val am = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val mem = ActivityManager.MemoryInfo()
        am.getMemoryInfo(mem)
        return DeviceMetrics(
            sdkInt = Build.VERSION.SDK_INT.toLong(),
            isLowRamDevice = am.isLowRamDevice,
            memoryClassMb = am.memoryClass.toLong(),
            totalMemMb = mem.totalMem / BYTES_PER_MB,
            mediaPerformanceClass = mediaPerformanceClass(),
            maxRefreshRateHz = maxRefreshRateHz(),
            thermalStatus = thermalStatus(),
        )
    }

    private fun mediaPerformanceClass(): Long? =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            Build.VERSION.MEDIA_PERFORMANCE_CLASS.toLong()
        } else {
            null
        }

    private fun maxRefreshRateHz(): Double? = runCatching {
        val dm = context.getSystemService(Context.DISPLAY_SERVICE) as DisplayManager
        dm.getDisplay(Display.DEFAULT_DISPLAY)
            ?.supportedModes
            ?.maxOfOrNull { it.refreshRate }
            ?.toDouble()
    }.getOrNull()

    private fun thermalStatus(): Long? =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            runCatching {
                val pm = context.getSystemService(Context.POWER_SERVICE) as PowerManager
                pm.currentThermalStatus.toLong()
            }.getOrNull()
        } else {
            null
        }

    private companion object {
        const val BYTES_PER_MB = 1024L * 1024L
    }
}
