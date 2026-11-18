package app.sargoshi

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.io.File
import java.security.KeyStore
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * Holds the 32-byte secret that keys every vault PIN record.
 *
 * The secret is random, encrypted with a non-exportable Android Keystore AES-GCM
 * key, and stored in the no-backup directory (also excluded by the backup rules).
 * [read] never creates a secret: if it is missing while PIN records exist, the
 * caller must report the vault as unavailable, not as "wrong PIN".
 */
class VaultPepperStore(context: Context) {
    private val file = File(context.noBackupFilesDir, FILE_NAME)

    @Synchronized
    fun exists(): Boolean = file.isFile

    /** Returns the stored secret or throws when it is missing or unreadable. */
    @Synchronized
    fun read(): ByteArray {
        check(file.isFile) { "missing" }
        val blob = file.readBytes()
        check(blob.size > IV_LEN + TAG_BYTES) { "corrupt" }
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(
            Cipher.DECRYPT_MODE,
            key(createIfMissing = false),
            GCMParameterSpec(TAG_BYTES * 8, blob, 0, IV_LEN),
        )
        val plain = cipher.doFinal(blob, IV_LEN, blob.size - IV_LEN)
        check(plain.size == PEPPER_LEN) { "corrupt" }
        return plain
    }

    /** Creates the secret if it does not exist yet, then returns it. */
    @Synchronized
    fun createIfMissing(): ByteArray {
        if (file.isFile) return read()
        val fresh = ByteArray(PEPPER_LEN).also { SecureRandom().nextBytes(it) }
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key(createIfMissing = true))
        val sealed = cipher.iv + cipher.doFinal(fresh)
        check(cipher.iv.size == IV_LEN) { "iv" }
        val tmp = File(file.parentFile, "$FILE_NAME.tmp")
        tmp.writeBytes(sealed)
        check(tmp.renameTo(file)) { "write" }
        return fresh
    }

    private fun key(createIfMissing: Boolean): SecretKey {
        val ks = KeyStore.getInstance(PROVIDER).apply { load(null) }
        (ks.getKey(ALIAS, null) as? SecretKey)?.let { return it }
        check(createIfMissing) { "key missing" }
        return generate(strongBox = true) ?: generate(strongBox = false)!!
    }

    private fun generate(strongBox: Boolean): SecretKey? = try {
        val spec = KeyGenParameterSpec.Builder(
            ALIAS,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setIsStrongBoxBacked(strongBox)
            .build()
        KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, PROVIDER)
            .apply { init(spec) }
            .generateKey()
    } catch (e: Exception) {
        if (strongBox) null else throw e
    }

    private companion object {
        const val PROVIDER = "AndroidKeyStore"
        const val ALIAS = "sargoshi.vault.pepper.v1"
        const val FILE_NAME = "vault_pepper.bin"
        const val TRANSFORMATION = "AES/GCM/NoPadding"
        const val IV_LEN = 12
        const val TAG_BYTES = 16
        const val PEPPER_LEN = 32
    }
}
