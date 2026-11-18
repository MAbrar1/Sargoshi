package app.sargoshi

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import java.io.File
import java.security.KeyStore
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/** Runs on a real Android Keystore (emulator or device), not a mock. */
@RunWith(AndroidJUnit4::class)
class VaultPepperStoreTest {
    private lateinit var context: Context
    private lateinit var file: File

    @Before
    fun clean() {
        context = InstrumentationRegistry.getInstrumentation().targetContext
        file = File(context.noBackupFilesDir, "vault_pepper.bin")
        wipe()
    }

    @After
    fun tearDown() = wipe()

    private fun wipe() {
        file.delete()
        File(file.parentFile, "vault_pepper.bin.tmp").delete()
        val ks = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        if (ks.containsAlias("sargoshi.vault.pepper.v1")) ks.deleteEntry("sargoshi.vault.pepper.v1")
    }

    private fun assertThrows(block: () -> Unit) {
        try {
            block()
            fail("expected an exception")
        } catch (e: Exception) {
            // expected
        }
    }

    @Test
    fun readWithoutSecretThrowsAndDoesNotCreate() {
        val store = VaultPepperStore(context)
        assertFalse(store.exists())
        assertThrows { store.read() }
        assertFalse(store.exists())
    }

    @Test
    fun createReturns32BytesAndIsStable() {
        val store = VaultPepperStore(context)
        val first = store.createIfMissing()
        assertEquals(32, first.size)
        assertTrue(store.exists())
        assertArrayEquals(first, store.read())
        assertArrayEquals(first, store.createIfMissing())
    }

    @Test
    fun aNewStoreInstanceReadsTheSameSecret() {
        val first = VaultPepperStore(context).createIfMissing()
        assertArrayEquals(first, VaultPepperStore(context).read())
    }

    @Test
    fun secretIsNotStoredInThePlain() {
        val secret = VaultPepperStore(context).createIfMissing()
        val stored = file.readBytes()
        assertNotEquals(secret.toList(), stored.toList())
        val asText = String(stored, Charsets.ISO_8859_1)
        assertFalse(asText.contains(String(secret, Charsets.ISO_8859_1)))
    }

    @Test
    fun aTamperedFileIsAnErrorNotANewSecret() {
        val store = VaultPepperStore(context)
        store.createIfMissing()
        val bytes = file.readBytes()
        bytes[bytes.size - 1] = (bytes[bytes.size - 1].toInt() xor 1).toByte()
        file.writeBytes(bytes)
        assertThrows { store.read() }
    }

    @Test
    fun aTruncatedFileIsAnError() {
        val store = VaultPepperStore(context)
        store.createIfMissing()
        file.writeBytes(byteArrayOf(1, 2, 3))
        assertThrows { store.read() }
    }

    @Test
    fun lostKeyWithSurvivingFileIsAnError() {
        val store = VaultPepperStore(context)
        store.createIfMissing()
        val ks = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        ks.deleteEntry("sargoshi.vault.pepper.v1")
        assertThrows { store.read() }
    }
}
