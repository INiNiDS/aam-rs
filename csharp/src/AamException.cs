using System;

namespace AamRs;

public sealed class AamException : Exception
{
    public AamException(string message) : base(message)
    {
    }
}

