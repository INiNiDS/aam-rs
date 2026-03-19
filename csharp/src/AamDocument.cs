using System;

namespace AamRs;

public sealed class AamDocument : IDisposable
{
    private SafeAamHandle? _handle;

    public AamDocument()
    {
        _handle = new SafeAamHandle();
    }

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

    public bool IsClosed => _handle is null || _handle.IsClosed || _handle.IsInvalid;

    public void Merge(string content)
    {
        CheckResult(AamNative.aam_merge(Handle, content));
    }

    public string? FindObj(string key)
    {
        return AamNative.TakeOwnedUtf8String(AamNative.aam_find_obj(Handle, key));
    }

    public string? FindKey(string value)
    {
        return AamNative.TakeOwnedUtf8String(AamNative.aam_find_key(Handle, value));
    }

    public string? FindDeep(string key)
    {
        return AamNative.TakeOwnedUtf8String(AamNative.aam_find_deep(Handle, key));
    }

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

