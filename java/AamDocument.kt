package com.aamrs

import java.io.File
import java.io.InputStream
import java.lang.ref.Cleaner
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.atomic.AtomicLong

/**
 * High-level JVM wrapper around the native aam-rs AAML parser.
 *
 * Create instances via the companion-object factories [parse] or [load];
 * the constructor is private.  The class implements [AutoCloseable], so it
 * can be used in a `use {}` block or try-with-resources statement, and the
 * underlying Rust object is freed immediately on [close].  Even if [close]
 * is never called, a JVM [Cleaner] guarantees that the native memory is
 * eventually released when the instance is garbage-collected.
 *
 * Every lookup method returns `null` when the requested key / value is not
 * found rather than throwing, mirroring the behaviour of the C and Python
 * bindings.
 *
 * ## Example
 * ```kotlin
 * AamDocument.parse("host = localhost\nport = 8080").use { doc ->
 *     println(doc.findObj("host"))  // "localhost"
 * }
 * ```
 *
 * ## Memory ownership
 * Strings returned by this class are regular JVM [String] objects; the
 * native allocations are managed internally and are never leaked to the
 * caller.
 */
class AamDocument private constructor(private var nativePtr: Long) : AutoCloseable {
    private val cleanable = CLEANER.register(this, NativeResource(nativePtr))

    private class NativeResource(ptr: Long) : Runnable {
        private val ptr = AtomicLong(ptr)

        override fun run() {
            val value = ptr.getAndSet(0L)
            if (value != 0L) {
                AamNative.destroy(value)
            }
        }
    }

    companion object {
        private val CLEANER: Cleaner = Cleaner.create()

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
                throw UnsupportedOperationException(
                    "Unsupported OS/Arch: $osPrefix-$archPrefix. " +
                    "Native library not found in JAR at $resourcePath"
                )
            }

            val tempFile = File.createTempFile("libaam_rs_", extension)
            tempFile.deleteOnExit()

            inputStream.use { input ->
                Files.copy(input, tempFile.toPath(), StandardCopyOption.REPLACE_EXISTING)
            }

            System.load(tempFile.absolutePath)
        }

        /**
         * Parses an AAML string and returns a new [AamDocument].
         *
         * @param content AAML text to parse (UTF-8).
         * @return A fully initialised [AamDocument].
         * @throws IllegalStateException if the content contains a parse error.
         */
        @JvmStatic
        fun parse(content: String): AamDocument {
            val ptr = AamNative.parse(content)
            if (ptr == 0L) throw IllegalStateException("Failed to parse AAML content")
            return AamDocument(ptr)
        }

        /**
         * Loads an AAML file from disk and returns a new [AamDocument].
         *
         * Directives such as `@import` inside the file are resolved relative
         * to the file's own directory.
         *
         * @param path Absolute or relative path to the `.aam` file.
         * @return A fully initialised [AamDocument].
         * @throws IllegalStateException if the file cannot be read or contains a parse error.
         */
        @JvmStatic
        fun load(path: String): AamDocument {
            val ptr = AamNative.load(path)
            if (ptr == 0L) throw IllegalStateException("Failed to load AAML file: $path")
            return AamDocument(ptr)
        }
    }

    /**
     * Merges additional AAML text into this document **without** resetting
     * keys that were already loaded.  Keys present in [content] override
     * existing values (child-wins semantics).
     *
     * @param content AAML text to merge.
     * @throws IllegalStateException if [close] has been called or parsing fails.
     */
    fun merge(content: String) {
        AamNative.merge(checkPtr(), content)
    }

    /**
     * Forward lookup: returns the value stored under [key], or performs a
     * reverse lookup (value → key) when no direct entry exists.
     *
     * @param key The key to look up.
     * @return The associated value, or `null` if not found.
     * @throws IllegalStateException if [close] has been called.
     */
    fun findObj(key: String): String? = AamNative.findObj(checkPtr(), key)

    /**
     * Reverse lookup: finds the *key* whose stored value equals [value].
     *
     * @param value The value to search for.
     * @return The key that maps to [value], or `null` if none exists.
     * @throws IllegalStateException if [close] has been called.
     */
    fun findKey(value: String): String? = AamNative.findKey(checkPtr(), value)

    /**
     * Deep / chain lookup: follows the chain `key → value → key` repeatedly
     * until a terminal (non-key) value is reached or a cycle is detected.
     *
     * For example, given `a = b`, `b = c`, `c = result`:
     * `findDeep("a")` returns `"result"`.
     *
     * @param key The starting key of the chain.
     * @return The terminal value, or `null` if the chain is empty.
     * @throws IllegalStateException if [close] has been called.
     */
    fun findDeep(key: String): String? = AamNative.findDeep(checkPtr(), key)

    /**
     * Looks up [key] and parses its value as a homogeneous list
     * `[item1, item2, ...]`.
     *
     * @param key The key whose value is a list literal.
     * @return An immutable [List] of string items, or `null` if the key is
     *         absent or its value is not a list literal.
     * @throws IllegalStateException if [close] has been called.
     */
    fun findList(key: String): List<String>? = AamNative.findList(checkPtr(), key)?.toList()

    /**
     * Looks up [key] and parses its value as an inline object
     * `{ k = v, ... }`.
     *
     * @param key The key whose value is an object literal.
     * @return An immutable [Map] of string pairs, or `null` if the key is
     *         absent or its value is not an object literal.
     * @throws IllegalStateException if [close] has been called.
     */
    fun findObject(key: String): Map<String, String>? = AamNative.findObject(checkPtr(), key)

    /**
     * Backward-compatible alias for [findObj].
     *
     * @param key The key to look up.
     * @return The associated value, or `null` if not found.
     */
    fun find(key: String): String? = findObj(key)

    /**
     * Releases the underlying Rust object immediately.
     *
     * After this call all methods throw [IllegalStateException].  Calling
     * [close] more than once is safe (idempotent).
     */
    override fun close() {
        if (nativePtr != 0L) {
            cleanable.clean()
            nativePtr = 0L
        }
    }

    private fun checkPtr(): Long {
        if (nativePtr == 0L) throw IllegalStateException("AamDocument is closed")
        return nativePtr
    }
}
