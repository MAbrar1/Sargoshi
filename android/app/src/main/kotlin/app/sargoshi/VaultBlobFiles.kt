package app.sargoshi

import android.content.Context
import java.io.File
import java.io.FileOutputStream

/**
 * Stores opaque, already-encrypted vault blobs (the wrapped master key and
 * sealed items) in the no-backup directory. Names look like `real/master` or
 * `real/item/note/a1` and are validated strictly, so a name can never escape
 * the directory. Writes are atomic and synced before the rename.
 */
class VaultBlobFiles(context: Context) {
    private val dir = File(context.noBackupFilesDir, "vault_blobs")

    @Synchronized
    fun read(name: String): ByteArray? {
        val f = file(name)
        return if (f.isFile) f.readBytes() else null
    }

    @Synchronized
    fun write(name: String, bytes: ByteArray) {
        val target = file(name)
        check(dir.isDirectory || dir.mkdirs()) { "dir" }
        val tmp = File(dir, target.name + TMP_SUFFIX)
        FileOutputStream(tmp).use { out ->
            out.write(bytes)
            out.fd.sync()
        }
        check(tmp.renameTo(target)) { "write" }
    }

    @Synchronized
    fun delete(name: String) {
        file(name).delete()
    }

    /** Names starting with [prefix]. The prefix is only compared, never used as a path. */
    @Synchronized
    fun list(prefix: String): List<String> {
        require(PREFIX.matches(prefix)) { "prefix" }
        val files = dir.listFiles() ?: return emptyList()
        return files
            .map { it.name }
            .filter { it.endsWith(SUFFIX) }
            .map { it.removeSuffix(SUFFIX).replace('.', '/') }
            .filter { NAME.matches(it) && it.startsWith(prefix) }
    }

    private fun file(name: String): File {
        require(NAME.matches(name)) { "name" }
        // No allowed segment contains '.', so this mapping is reversible.
        return File(dir, name.replace('/', '.') + SUFFIX)
    }

    companion object {
        private const val SUFFIX = ".blob"
        private const val TMP_SUFFIX = ".tmp"
        private val NAME =
            Regex("^[a-z0-9]{1,16}/(master|item/[a-z0-9_]{1,32}/[A-Za-z0-9_-]{1,64})$")
        private val PREFIX = Regex("^[A-Za-z0-9_/-]{0,120}$")
    }
}
