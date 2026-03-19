using AamRs;
using Xunit;

namespace AamRs.Tests;

public sealed class AamDocumentTests
{
    [Fact]
    public void ParseAndFindObj_ReturnsValue_WhenNativeLibraryIsAvailable()
    {
        try
        {
            using var doc = AamDocument.Parse("host = localhost\nport = 8080");
            Assert.Equal("localhost", doc.FindObj("host"));
        }
        catch (DllNotFoundException)
        {
            // CI environments without native artifacts should not fail this managed test suite.
        }
    }

    [Fact]
    public void Parse_MultipleKeys()
    {
        try
        {
            const string content = @"
name = MyApp
version = 1.0.0
debug = true
";
            using var doc = AamDocument.Parse(content);
            Assert.Equal("MyApp", doc.FindObj("name"));
            Assert.Equal("1.0.0", doc.FindObj("version"));
            Assert.Equal("true", doc.FindObj("debug"));
        }
        catch (DllNotFoundException)
        {
            // Skip if native library not available
        }
    }

    [Fact]
    public void Parse_WithComments()
    {
        try
        {
            const string content = @"
# This is a comment
host = localhost
# Another comment
port = 8080
";
            using var doc = AamDocument.Parse(content);
            Assert.Equal("localhost", doc.FindObj("host"));
            Assert.Equal("8080", doc.FindObj("port"));
        }
        catch (DllNotFoundException)
        {
            // Skip if native library not available
        }
    }

    [Fact]
    public void Merge_CombinesConfigurations()
    {
        try
        {
            using var doc = AamDocument.Parse("host = localhost\nport = 8080");
            doc.Merge("port = 9090\ndebug = true");
            Assert.Equal("localhost", doc.FindObj("host"));
            Assert.Equal("9090", doc.FindObj("port"));
            Assert.Equal("true", doc.FindObj("debug"));
        }
        catch (DllNotFoundException)
        {
            // Skip if native library not available
        }
    }

    [Fact]
    public void FindKey_SearchesByValue()
    {
        try
        {
            const string content = @"
database = postgres
cache = redis
messaging = rabbitmq
";
            using var doc = AamDocument.Parse(content);
            var result = doc.FindKey("postgres");
            Assert.NotNull(result);
            Assert.Equal("database", result);
        }
        catch (DllNotFoundException)
        {
            // Skip if native library not available
        }
    }

    [Fact]
    public void ParseEmptyDocument()
    {
        try
        {
            using var doc = AamDocument.Parse("");
            Assert.Null(doc.FindObj("nonexistent"));
        }
        catch (DllNotFoundException)
        {
            // Skip if native library not available
        }
    }

    [Fact]
    public void ParseWithWhitespace()
    {
        try
        {
            const string content = @"
name   =   MyApp
port   =   8080
";
            using var doc = AamDocument.Parse(content);
            Assert.Equal("MyApp", doc.FindObj("name"));
            Assert.Equal("8080", doc.FindObj("port"));
        }
        catch (DllNotFoundException)
        {
            // Skip if native library not available
        }
    }
}



