using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace AamCsharp;

public readonly struct SchemaField
{
    public string Name { get; }
    public string TypeName { get; }
    public bool Optional { get; }

    private SchemaField(string name, string typeName, bool optional)
    {
        Name = name;
        TypeName = typeName;
        Optional = optional;
    }

    public static SchemaField Required(string name, string typeName) => new(name, typeName, false);
    public static SchemaField OptionalField(string name, string typeName) => new(name, typeName, true);

    public string ToAam() => Optional ? $"{Name}*: {TypeName}" : $"{Name}: {TypeName}";
}

public sealed class AamBuilder
{
    private readonly StringBuilder _buffer;

    public AamBuilder() : this(0)
    {
    }

    public AamBuilder(int capacity)
    {
        _buffer = capacity > 0 ? new StringBuilder(capacity) : new StringBuilder();
    }

    public AamBuilder AddLine(string key, string value)
    {
        PushSeparator();
        _buffer.Append(key).Append(" = ").Append(value);
        return this;
    }

    public AamBuilder Comment(string text)
    {
        PushSeparator();
        _buffer.Append("# ").Append(text);
        return this;
    }

    public AamBuilder Schema(string name, IEnumerable<SchemaField> fields)
    {
        PushSeparator();
        _buffer.Append("@schema ").Append(name).Append(" { ");

        var first = true;
        foreach (var field in fields)
        {
            if (!first)
            {
                _buffer.Append(", ");
            }

            _buffer.Append(field.ToAam());
            first = false;
        }

        _buffer.Append(" }");
        return this;
    }

    public AamBuilder SchemaMultiline(string name, IEnumerable<SchemaField> fields)
    {
        PushSeparator();
        _buffer.Append("@schema ").Append(name).Append(" {");
        foreach (var field in fields)
        {
            _buffer.AppendLine();
            _buffer.Append("    ").Append(field.ToAam());
        }

        _buffer.AppendLine();
        _buffer.Append('}');
        return this;
    }

    public AamBuilder Derive(string path, IEnumerable<string> schemas)
    {
        PushSeparator();
        _buffer.Append("@derive ").Append(path);
        foreach (var schema in schemas)
        {
            _buffer.Append("::").Append(schema);
        }

        return this;
    }

    public AamBuilder Import(string path)
    {
        PushSeparator();
        _buffer.Append("@import ").Append(path);
        return this;
    }

    public AamBuilder TypeAlias(string alias, string typeName)
    {
        PushSeparator();
        _buffer.Append("@type ").Append(alias).Append(" = ").Append(typeName);
        return this;
    }

    public AamBuilder AddInline(string key, InlineObject obj)
    {
        AddLine(key, obj.ToString());
        return this;
    }

    public string Build() => _buffer.ToString();

    public void ToFile(string path)
    {
        File.WriteAllText(path, _buffer.ToString());
    }

    public override string ToString() => _buffer.ToString();

    private void PushSeparator()
    {
        if (_buffer.Length > 0)
        {
            _buffer.AppendLine();
        }
    }
}

/// Builds `{ key = value, ... }` inline object literals.
public sealed class InlineObject
{
    private readonly List<string> _pairs = new();

    public InlineObject Add(string key, string value)
    {
        _pairs.Add($"{key} = {value}");
        return this;
    }

    public override string ToString() => "{ " + string.Join(", ", _pairs) + " }";
}

/// Parse an inline object string into a string dictionary.
public static class AamInline
{
    public static Dictionary<string, string> ParseInlineToMap(string content)
    {
        unsafe
        {
            fixed (byte* contentPtr = System.Text.Encoding.UTF8.GetBytes(content + "\0"))
            {
                var result = AamNative.aam_parse_inline_to_map(contentPtr);
                if (result == null)
                    return new Dictionary<string, string>();

                var str = System.Runtime.InteropServices.Marshal.PtrToStringUTF8((nint)result);
                AamNative.aam_string_free(result);

                var dict = new Dictionary<string, string>();
                if (str == null) return dict;

                foreach (var line in str.Split('\n'))
                {
                    var parts = line.Split('=', 2);
                    if (parts.Length == 2)
                        dict[parts[0]] = parts[1];
                }
                return dict;
            }
        }
    }
}

