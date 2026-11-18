package app.sargoshi

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class VaultRecordFilesTest {
    private lateinit var files: VaultRecordFiles

    @Before
    fun setUp() {
        files = VaultRecordFiles(InstrumentationRegistry.getInstrumentation().targetContext)
        VaultRecordFiles.NAMES.forEach(files::delete)
    }

    @After
    fun tearDown() = VaultRecordFiles.NAMES.forEach(files::delete)

    @Test
    fun missingRecordIsNull() = assertNull(files.read("real"))

    @Test
    fun writeThenReadRoundTripsAndOverwrites() {
        files.write("real", byteArrayOf(1, 2, 3))
        assertArrayEquals(byteArrayOf(1, 2, 3), files.read("real"))
        files.write("real", byteArrayOf(9))
        assertArrayEquals(byteArrayOf(9), files.read("real"))
        assertNull(files.read("decoy"))
    }

    @Test
    fun deleteRemovesTheRecord() {
        files.write("decoy", byteArrayOf(5))
        files.delete("decoy")
        assertNull(files.read("decoy"))
    }

    @Test(expected = IllegalArgumentException::class)
    fun unknownNamesAreRejected() {
        files.write("../escape", byteArrayOf(1))
    }
}
