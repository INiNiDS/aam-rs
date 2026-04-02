using System;
using System.Collections.Generic;
using System.IO;

namespace AamCsharp;

/// <summary>
/// Represents an AAM document and provides operations for parsing, loading, querying, formatting and merging data.
/// </summary>
public sealed unsafe class AamDocument : IDisposable
{
    private SafeAamHandle? _handle;
    private string _sourceSnapshot = string.Empty;

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
            document._sourceSnapshot = content;
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
            try
            {
                document._sourceSnapshot = File.ReadAllText(path);
            }
            catch
            {
                document._sourceSnapshot = string.Empty;
            }
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
    /// Finds a value by key using direct key lookup.
    /// </summary>
    public string? Get(string key)
    {
        return AamNative.TakeOwnedUtf8String(AamNative.aam_get(Handle, key));
    }

    /// <summary>
    /// Smart lookup: by key first, then by value fallback.
    /// </summary>
    public Dictionary<string, string> Find(string query)
    {
        var raw = AamNative.TakeOwnedUtf8String(AamNative.aam_find(Handle, query));
        return ParseLineMap(raw);
    }

    /// <summary>
    /// Finds keys that match a given value.
    /// </summary>
    public string[] ReverseSearch(string value)
    {
        var raw = AamNative.TakeOwnedUtf8String(AamNative.aam_reverse_search(Handle, value));
        if (string.IsNullOrWhiteSpace(raw))
        {
            return Array.Empty<string>();
        }

        return raw.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
    }

    /// <summary>
    /// Finds key-value pairs where keys contain the specified pattern.
    /// </summary>
    public Dictionary<string, string> DeepSearch(string pattern)
    {
        var raw = AamNative.TakeOwnedUtf8String(AamNative.aam_deep_search(Handle, pattern));
        return ParseLineMap(raw);
    }

    public string[] SchemaNames()
    {
        var raw = AamNative.TakeOwnedUtf8String(AamNative.aam_schema_names(Handle));
        if (string.IsNullOrWhiteSpace(raw))
        {
            return Array.Empty<string>();
        }

        return raw.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
    }

    public string[] TypeNames()
    {
        var raw = AamNative.TakeOwnedUtf8String(AamNative.aam_type_names(Handle));
        if (string.IsNullOrWhiteSpace(raw))
        {
            return Array.Empty<string>();
        }

        return raw.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
    }

    /// <summary>
    /// Finds a value by key.
    /// </summary>
    /// <param name="key">Key to search for.</param>
    /// <returns>The value for the key, or <see langword="null"/> if not found.</returns>
    public string? FindObj(string key)
    {
        var direct = Get(key);
        if (direct is not null)
        {
            return direct;
        }

        return FindKey(key);
    }

    /// <summary>
    /// Finds a key by its value.
    /// </summary>
    /// <param name="value">Value to search for.</param>
    /// <returns>The first matching key, or <see langword="null"/> if not found.</returns>
    public string? FindKey(string value)
    {
        var keys = ReverseSearch(value);
        return keys.Length > 0 ? keys[0] : null;
    }

    /// <summary>
    /// Resolves a value through chained key lookups until a terminal value is reached.
    /// </summary>
    /// <param name="key">Starting key for deep resolution.</param>
    /// <returns>The resolved terminal value, or <see langword="null"/> if resolution fails.</returns>
    public string? FindDeep(string key)
    {
        var seen = new HashSet<string>(StringComparer.Ordinal);
        var current = key;

        while (true)
        {
            if (!seen.Add(current))
            {
                return current;
            }

            var next = Get(current);
            if (next is null)
            {
                return current == key ? null : current;
            }

            current = next;
        }
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

    private static Dictionary<string, string> ParseLineMap(string? raw)
    {
        var map = new Dictionary<string, string>(StringComparer.Ordinal);
        if (string.IsNullOrWhiteSpace(raw))
        {
            return map;
        }

        var lines = raw.Split('\n', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        foreach (var line in lines)
        {
            var idx = line.IndexOf('=');
            if (idx <= 0 || idx == line.Length - 1)
            {
                continue;
            }

            var k = line[..idx];
            var v = line[(idx + 1)..];
            map[k] = v;
        }

        return map;
    }
}
