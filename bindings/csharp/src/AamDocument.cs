using System;
using System.Collections.Generic;

namespace AamCsharp;

/// <summary>
/// Represents an AAM document and provides operations for parsing, loading, querying and formatting data.
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
    /// Splits a multi-section AAM text into a dictionary of builders keyed by section filename.
    /// </summary>
    public static Dictionary<string, AamBuilder> SplitAam(string content)
    {
        var result = new Dictionary<string, AamBuilder>(StringComparer.Ordinal);
        string? currentName = null;
        AamBuilder? currentBuilder = null;

        foreach (var rawLine in content.Split('\n'))
        {
            var line = rawLine.Trim();
            if (string.IsNullOrEmpty(line))
            {
                continue;
            }

            if (ParseSectionHeader(line) is string header)
            {
                if (currentName is not null && currentBuilder is not null)
                {
                    result[currentName] = currentBuilder;
                }

                currentName = header;
                currentBuilder = new AamBuilder();
                continue;
            }

            if (currentName is null || currentBuilder is null)
            {
                continue;
            }

            if (ParseAssignment(line) is { } assignment)
            {
                currentBuilder.AddLine(assignment.key, assignment.value);
            }
        }

        if (currentName is not null && currentBuilder is not null)
        {
            result[currentName] = currentBuilder;
        }

        return result;
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
    /// Reconstructs a @schema directive from a list of AAM content strings.
    /// </summary>
    /// <param name="name">Schema name.</param>
    /// <param name="contents">Individual AAM configuration content strings to analyze.</param>
    /// <returns>A formatted @schema string.</returns>
    /// <exception cref="AamException">Thrown when reconstruction fails.</exception>
    public string ReconstructSchema(string name, params string[] contents)
    {
        AamNative.aam_reconstruct_clear(Handle);
        foreach (var content in contents)
        {
            var rc = AamNative.aam_reconstruct_push(Handle, content);
            if (rc != 0)
            {
                var errPtr = AamNative.aam_last_error(Handle);
                var message = AamNative.BorrowUtf8String(errPtr) ?? "Native push failed";
                throw new AamException(message);
            }
        }

        var ptr = AamNative.aam_reconstruct_schema(Handle, name);
        if (ptr == null)
        {
            var errPtr = AamNative.aam_last_error(Handle);
            var message = AamNative.BorrowUtf8String(errPtr) ?? "Native schema reconstruction failed";
            throw new AamException(message);
        }
        return AamNative.TakeOwnedUtf8String(ptr)!;
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

    private static string? ParseSectionHeader(string line)
    {
        if (!line.StartsWith('#'))
        {
            return null;
        }

        var rest = line[1..].Trim();
        return rest.EndsWith(".aam", StringComparison.Ordinal) ? rest : null;
    }

    private static (string key, string value)? ParseAssignment(string line)
    {
        var idx = line.IndexOf('=');
        if (idx <= 0)
        {
            return null;
        }

        var key = line[..idx].Trim();
        if (string.IsNullOrEmpty(key))
        {
            return null;
        }

        var value = line[(idx + 1)..].Trim();
        return (key, value);
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
