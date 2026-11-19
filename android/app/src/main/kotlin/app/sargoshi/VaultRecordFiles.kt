package app.sargoshi

import android.content.Context
import java.io.File

/**
 * Stores the vault PIN verifier records ("real" and "decoy") in the no-backup
 * directory. The records are useless without the Keystore secret, but they are
 * still kept out of backups. Writes are atomic.
 */
class VaultRecordFiles(context: Context) {
    private val dir = File(context.noBackupFilesDir, "vault")

    @Synchronized
    fun read(name: String): ByteArray? {
        val f = file(name)
        return if (f.isFile) f.readBytes() else null
    }

    @Synchronized
    fun write(name: String, bytes: ByteArray) {
        check(dir.isDirectory || dir.mkdirs()) { "dir" }
        val target = file(name)
        val tmp = File(dir, "$name.tmp")
        tmp.writeBytes(bytes)
        check(tmp.renameTo(target)) { "write" }
    }

    @Synchronized
    fun delete(name: String) {
        file(name).delete()
    }

    private fun file(name: String): File {
        require(name in NAMES) { "name" }
        return File(dir, "$name.rec")
    }

    companion object {
        val NAMES = setOf("real", "decoy", "attempts")
    }
}
