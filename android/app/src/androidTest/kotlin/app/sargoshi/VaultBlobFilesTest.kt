package app.sargoshi

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class VaultBlobFilesTest {
    private lateinit var files: VaultBlobFiles
    private val names = listOf("real/master", "real/item/note/a1", "real/item/note/b-2", "real/item/photo/p1", "decoy/master")

    @Before
    fun setUp() {
        files = VaultBlobFiles(InstrumentationRegistry.getInstrumentation().targetContext)
        names.forEach(files::delete)
    }

    @After
    fun tearDown() = names.forEach(files::delete)

    @Test
    fun missingBlobIsNull() = assertNull(files.read("real/master"))

    @Test
    fun writeThenReadRoundTripsAndOverwrites() {
        files.write("real/item/note/a1", byteArrayOf(1, 2, 3))
        assertArrayEquals(byteArrayOf(1, 2, 3), files.read("real/item/note/a1"))
        files.write("real/item/note/a1", byteArrayOf(9))
        assertArrayEquals(byteArrayOf(9), files.read("real/item/note/a1"))
        assertNull(files.read("real/item/note/b-2"))
    }

    @Test
    fun deleteRemovesTheBlob() {
        files.write("decoy/master", byteArrayOf(5))
        files.delete("decoy/master")
        assertNull(files.read("decoy/master"))
    }

    @Test
    fun listReturnsOnlyMatchingNamesWithTheirOriginalSlashes() {
        names.forEach { files.write(it, byteArrayOf(1)) }
        assertEquals(
            listOf("real/item/note/a1", "real/item/note/b-2"),
            files.list("real/item/note/").sorted(),
        )
        assertEquals(listOf("real/item/photo/p1"), files.list("real/item/photo/"))
        assertEquals(listOf("decoy/master"), files.list("decoy/"))
        assertEquals(emptyList<String>(), files.list("nobody/"))
    }

    @Test
    fun hostileNamesAreRejected() {
        val bad = listOf("../escape", "real/../x", "real/item/note/a.b", "real/item/Note/a1", "/real/master", "real/master/", "real//master", "")
        bad.forEach { name ->
            var rejected = false
            try {
                files.write(name, byteArrayOf(1))
            } catch (e: IllegalArgumentException) {
                rejected = true
            }
            assertEquals("should reject '$name'", true, rejected)
        }
    }

    @Test(expected = IllegalArgumentException::class)
    fun hostilePrefixIsRejected() {
        files.list("../")
    }
}
