package com.aamrs

/**
 * Low-level JNI bridge to the native aam-rs library.
 *
 * This object exposes the raw Rust FFI surface.  Prefer using [AamDocument],
 * which wraps these calls with lifecycle management and a friendlier API.
 *
 * ## Memory model
 * - [parse] and [load] allocate a `Box<AAML>` on the Rust heap and return
 *   its address as a `Long` handle.
 * - The handle remains valid until [destroy] is called.
 * - All lookup functions return JVM `String` objects; no native memory is
 *   transferred to the caller.
 * - [destroy] must be called exactly once for every handle returned by
 *   [parse] or [load].  [AamDocument] does this automatically via its
 *   `Cleaner`.
 */
object AamNative {

    /**
     * Parses AAML text and returns an opaque native handle.
     *
     * @param content UTF-8 AAML text.
     * @return Non-zero native pointer on success; `0` on failure (a Java
     *         exception is thrown in that case).
     */
    @JvmStatic external fun parse(content: String): Long

    /**
     * Loads an AAML file from disk and returns an opaque native handle.
     *
     * @param path Path to the `.aam` file (UTF-8).
     * @return Non-zero native pointer on success; `0` on failure.
     */
    @JvmStatic external fun load(path: String): Long

    /**
     * Merges additional AAML text into an existing handle **in-place**.
     *
     * Keys already stored in [ptr] are preserved; keys in [content] override
     * conflicting entries.
     *
     * @param ptr Valid native handle returned by [parse] or [load].
     * @param content UTF-8 AAML text to merge.
     */
    @JvmStatic external fun merge(ptr: Long, content: String)

    /**
     * Forward lookup: returns the value for [key], or `null`.
     *
     * Also performs a reverse lookup (value → key) when the key is not
     * found directly.
     *
     * @param ptr Valid native handle.
     * @param key Key to look up.
     * @return The associated value, or `null`.
     */
    @JvmStatic external fun findObj(ptr: Long, key: String): String?

    /**
     * Reverse lookup: finds the key whose value equals [value].
     *
     * @param ptr Valid native handle.
     * @param value Value to search for.
     * @return The matching key, or `null`.
     */
    @JvmStatic external fun findKey(ptr: Long, value: String): String?

    /**
     * Deep / chain lookup: follows `key → value → key` until a terminal
     * value or a cycle is detected.
     *
     * @param ptr Valid native handle.
     * @param key Starting key of the chain.
     * @return Terminal value of the chain, or `null`.
     */
    @JvmStatic external fun findDeep(ptr: Long, key: String): String?

    /**
     * Parses the value of [key] as a list `[a, b, c]`.
     *
     * @param ptr Valid native handle.
     * @param key Key whose value is a list literal.
     * @return Array of string items, or `null`.
     */
    @JvmStatic external fun findList(ptr: Long, key: String): Array<String>?

    /**
     * Parses the value of [key] as an inline object `{ k = v, ... }`.
     *
     * @param ptr Valid native handle.
     * @param key Key whose value is an object literal.
     * @return Map of string pairs, or `null`.
     */
    @JvmStatic external fun findObject(ptr: Long, key: String): Map<String, String>?

    /**
     * Frees the native Rust object referenced by [ptr].
     *
     * Must be called exactly once per handle.  Passing `0` is a no-op.
     *
     * @param ptr Handle to release.
     */
    @JvmStatic external fun destroy(ptr: Long)
}
