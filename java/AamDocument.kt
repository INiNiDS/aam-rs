package com.aamrs

import java.io.File
import java.io.InputStream
import java.nio.file.Files
import java.nio.file.StandardCopyOption

class AamDocument private constructor(private var nativePtr: Long) : AutoCloseable {

    companion object {
        init {
            loadNativeLibrary()
        }

        private fun loadNativeLibrary() {
            val osName = System.getProperty("os.name").lowercase()
            val osArch = System.getProperty("os.arch").lowercase()

            val osPrefix = when {
                osName.contains("win") -> "windows"
                osName.contains("mac") -> "macos"
                else -> "linux"
            }

            val archPrefix = when {
                osArch.contains("aarch64") || osArch.contains("arm64") -> "aarch64"
                else -> "x86_64"
            }

            val extension = when (osPrefix) {
                "windows" -> ".dll"
                "macos" -> ".dylib"
                else -> ".so"
            }

            val libName = if (osPrefix == "windows") "aam_rs$extension" else "libaam_rs$extension"
            val resourcePath = "/natives/$osPrefix-$archPrefix/$libName"

            val inputStream: InputStream? = AamDocument::class.java.getResourceAsStream(resourcePath)
            if (inputStream == null) {
                throw UnsupportedOperationException("Unsupported OS/Arch: $osPrefix-$archPrefix. Native library not found in JAR at $resourcePath")
            }

            val tempFile = File.createTempFile("libaam_rs_", extension)
            tempFile.deleteOnExit()

            inputStream.use { input ->
                Files.copy(input, tempFile.toPath(), StandardCopyOption.REPLACE_EXISTING)
            }

            System.load(tempFile.absolutePath)
        }
    }

    private external fun findObj(ptr: Long, key: String): String?
    private external fun findDeep(ptr: Long, path: String): String?
    private external fun findListNative(ptr: Long, key: String): Array<String>?
    private external fun findObjectNative(ptr: Long, key: String): Map<String, String>?
    private external fun destroy(ptr: Long)

    // Методы экземпляра
    fun find(key: String): String? {
        checkPtr()
        return findObj(nativePtr, key)
    }

    fun findDeep(path: String): String? {
        checkPtr()
        return findDeep(nativePtr, path)
    }

    fun findList(key: String): List<String>? {
        checkPtr()
        return findListNative(nativePtr, key)?.toList()
    }

    fun findObject(key: String): Map<String, String>? {
        checkPtr()
        return findObjectNative(nativePtr, key)
    }

    override fun close() {
        if (nativePtr != 0L) {
            destroy(nativePtr)
            nativePtr = 0L
        }
    }

    private fun checkPtr() {
        if (nativePtr == 0L) throw IllegalStateException("AamDocument is closed")
    }
}
