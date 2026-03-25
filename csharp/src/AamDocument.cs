using System;

namespace AamCsharp;

/// <summary>
/// Represents an AAM document and provides operations for parsing, loading, querying, formatting and merging data.
/// </summary>
public sealed unsafe class AamDocument : IDisposable
{
    private SafeAamHandle? _handle;

    /// <summary>
    /// Initializes a new empty AAM document handle.
    /// </summary>
    public AamDocument()
    {
        _handle = new SafeAamHandle();
    }

    /// <summary>
    /// Parses AAM content from a string and returns a new document instance.
    /// </summary>
    /// <param name="content">AAM text to parse.</param>
    /// <returns>A new <see cref="AamDocument"/> instance containing parsed data.</returns>
    /// <exception cref="AamException">Thrown when native parsing fails.</exception>
    public static AamDocument Parse(string content)
    {
        var document = new AamDocument();
        try
        {
            document.CheckResult(AamNative.aam_parse(document.Handle, content));
            return document;
        }
        catch
        {
            document.Dispose();
            throw;
        }
    }

    /// <summary>
    /// Loads an AAM file from disk and returns a new document instance.
    /// </summary>
    /// <param name="path">Path to the AAM file.</param>
    /// <returns>A new <see cref="AamDocument"/> instance containing loaded data.</returns>
    /// <exception cref="AamException">Thrown when native loading fails.</exception>
    public static AamDocument Load(string path)
    {
        var document = new AamDocument();
        try
        {
            document.CheckResult(AamNative.aam_load(document.Handle, path));
            return document;
        }
        catch
        {
            document.Dispose();
            throw;
        }
    }

    /// <summary>
    /// Gets a value indicating whether the document has been disposed.
    /// </summary>
    public bool IsClosed => _handle is null || _handle.IsClosed || _handle.IsInvalid;

    /// <summary>
    /// Formats an AAM string using standardized rules.
    /// </summary>
    /// <param name="content">AAM text to format.</param>
    /// <returns>Formatted AAM string.</returns>
    /// <exception cref="AamException">Thrown when native formatting fails.</exception>
    public string Format(string content)
    {
        var ptr = AamNative.aam_format(Handle, content);
        if (ptr == null)
        {
            var errPtr = AamNative.aam_last_error(Handle);
            var message = AamNative.BorrowUtf8String(errPtr) ?? "Native formatting failed";
            throw new AamException(message);
        }
        return AamNative.TakeOwnedUtf8String(ptr)!;
    }

    /// <summary>
    /// Merges AAM content into the current document.
    /// </summary>
    /// <param name="content">AAM text to merge.</param>
    /// <exception cref="AamException">Thrown when native merge fails.</exception>
    public void Merge(string content)
    {
        CheckResult(AamNative.aam_merge(Handle, content));
    }

    /// <summary>
    /// Finds a value by key.
    /// </summary>
    /// <param name="key">Key to search for.</param>
    /// <returns>The value for the key, or <see langword="null"/> if not found.</returns>
    public string? FindObj(string key)
    {
        return AamNative.TakeOwnedUtf8String(AamNative.aam_find_obj(Handle, key));
    }

    /// <summary>
    /// Finds a key by its value.
    /// </summary>
    /// <param name="value">Value to search for.</param>
    /// <returns>The first matching key, or <see langword="null"/> if not found.</returns>
    public string? FindKey(string value)
    {
        return AamNative.TakeOwnedUtf8String(AamNative.aam_find_key(Handle, value));
    }

    /// <summary>
    /// Resolves a value through chained key lookups until a terminal value is reached.
    /// </summary>
    /// <param name="key">Starting key for deep resolution.</param>
    /// <returns>The resolved terminal value, or <see langword="null"/> if resolution fails.</returns>
    public string? FindDeep(string key)
    {
        return AamNative.TakeOwnedUtf8String(AamNative.aam_find_deep(Handle, key));
    }

    /// <summary>
    /// Releases native resources associated with this document.
    /// </summary>
    public void Dispose()
    {
        _handle?.Dispose();
        _handle = null;
        GC.SuppressFinalize(this);
    }

    private SafeAamHandle Handle
    {
        get
        {
            if (_handle is null || _handle.IsClosed || _handle.IsInvalid)
            {
                throw new ObjectDisposedException(nameof(AamDocument), "AamDocument is closed");
            }

            return _handle;
        }
    }

    private void CheckResult(int result)
    {
        if (result == 0)
        {
            return;
        }

        var errPtr = AamNative.aam_last_error(Handle);
        var message = AamNative.BorrowUtf8String(errPtr) ?? "Native operation failed";
        throw new AamException(message);
    }
}
