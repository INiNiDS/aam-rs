using System;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace AamCsharp;

internal sealed unsafe class SafeAamHandle : SafeHandleZeroOrMinusOneIsInvalid
{
    public SafeAamHandle() : base(true)
    {
        SetHandle((IntPtr)AamNative.aam_new());
        if (IsInvalid)
        {
            throw new InvalidOperationException("Failed to allocate native AAML handle");
        }
    }

    protected override bool ReleaseHandle()
    {
        AamNative.aam_free((AamlHandle*)handle);
        return true;
    }
}

internal static unsafe partial class AamNative
{
    internal static int aam_parse(SafeAamHandle handle, string content)
    {
        var utf8 = ToNullTerminatedUtf8(content);
        fixed (byte* ptr = utf8)
        {
            return aam_parse((AamlHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static int aam_load(SafeAamHandle handle, string path)
    {
        var utf8 = ToNullTerminatedUtf8(path);
        fixed (byte* ptr = utf8)
        {
            return aam_load((AamlHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static int aam_merge(SafeAamHandle handle, string content)
    {
        var utf8 = ToNullTerminatedUtf8(content);
        fixed (byte* ptr = utf8)
        {
            return aam_merge((AamlHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_find_obj(SafeAamHandle handle, string key)
    {
        var utf8 = ToNullTerminatedUtf8(key);
        fixed (byte* ptr = utf8)
        {
            return aam_find_obj((AamlHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_find_key(SafeAamHandle handle, string value)
    {
        var utf8 = ToNullTerminatedUtf8(value);
        fixed (byte* ptr = utf8)
        {
            return aam_find_key((AamlHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_find_deep(SafeAamHandle handle, string key)
    {
        var utf8 = ToNullTerminatedUtf8(key);
        fixed (byte* ptr = utf8)
        {
            return aam_find_deep((AamlHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_last_error(SafeAamHandle handle)
    {
        return aam_last_error((AamlHandle*)handle.DangerousGetHandle());
    }

    internal static string? BorrowUtf8String(byte* ptr)
    {
        if (ptr == null)
        {
            return null;
        }

        return Marshal.PtrToStringUTF8((IntPtr)ptr);
    }

    internal static string? TakeOwnedUtf8String(byte* ptr)
    {
        if (ptr == null)
        {
            return null;
        }

        try
        {
            return Marshal.PtrToStringUTF8((IntPtr)ptr);
        }
        finally
        {
            aam_string_free(ptr);
        }
    }

    private static byte[] ToNullTerminatedUtf8(string value)
    {
        var utf8 = Encoding.UTF8.GetBytes(value);
        var nullTerminated = new byte[utf8.Length + 1];
        Buffer.BlockCopy(utf8, 0, nullTerminated, 0, utf8.Length);
        return nullTerminated;
    }
}

