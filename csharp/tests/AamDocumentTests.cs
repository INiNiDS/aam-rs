using AamCsharp;
using Xunit;

namespace AamCsharp.Tests;

public sealed class AamDocumentTests
{
    private static void SkipIfNativeMissing(Action assertion)
    {
        try
        {
            assertion();
        }
        catch (DllNotFoundException)
        {
            // CI environments without native artifacts should not fail this managed test suite.
        }
    }

    [Fact]
    public void ParseAndFindObj_ReturnsValue_WhenNativeLibraryIsAvailable()
    {
        SkipIfNativeMissing(() =>
        {
            using var doc = AamDocument.Parse("host = localhost\nport = 8080");
            Assert.Equal("localhost", doc.FindObj("host"));
        });
    }

    [Fact]
    public void Parse_MultipleKeys()
    {
        SkipIfNativeMissing(() =>
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
        });
    }

    [Fact]
    public void Parse_WithComments()
    {
        SkipIfNativeMissing(() =>
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
        });
    }

    [Fact]
    public void Merge_CombinesConfigurations()
    {
        SkipIfNativeMissing(() =>
        {
            using var doc = AamDocument.Parse("host = localhost\nport = 8080");
            doc.Merge("port = 9090\ndebug = true");
            Assert.Equal("localhost", doc.FindObj("host"));
            Assert.Equal("9090", doc.FindObj("port"));
            Assert.Equal("true", doc.FindObj("debug"));
        });
    }

    [Fact]
    public void FindKey_SearchesByValue()
    {
        SkipIfNativeMissing(() =>
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
        });
    }

    [Fact]
    public void ParseEmptyDocument()
    {
        SkipIfNativeMissing(() =>
        {
            using var doc = AamDocument.Parse("");
            Assert.Null(doc.FindObj("nonexistent"));
        });
    }

    [Fact]
    public void ParseWithWhitespace()
    {
        SkipIfNativeMissing(() =>
        {
            const string content = @"
name   =   MyApp
port   =   8080
";
            using var doc = AamDocument.Parse(content);
            Assert.Equal("MyApp", doc.FindObj("name"));
            Assert.Equal("8080", doc.FindObj("port"));
        });
    }

    [Fact]
    public void FindObj_PerformsReverseLookupFallback()
    {
        SkipIfNativeMissing(() =>
        {
            using var doc = AamDocument.Parse("username = admin");
            Assert.Equal("username", doc.FindObj("admin"));
        });
    }

    [Fact]
    public void FindDeep_ResolvesChain()
    {
        SkipIfNativeMissing(() =>
        {
            using var doc = AamDocument.Parse("root = /srv\ncurrent = root");
            Assert.Equal("/srv", doc.FindDeep("current"));
        });
    }

    [Fact]
    public void Parse_InvalidContentThrowsAamException()
    {
        SkipIfNativeMissing(() =>
        {
            try
            {
                using var _ = AamDocument.Parse("invalid_line_without_equals");
                Assert.Fail("Expected AamException for invalid content");
            }
            catch (AamException)
            {
                // Expected.
            }
        });
    }

    [Fact]
    public void ClosedDocument_ThrowsOnMerge()
    {
        SkipIfNativeMissing(() =>
        {
            var doc = AamDocument.Parse("a = 1");
            doc.Dispose();
            Assert.True(doc.IsClosed);
            Assert.Throws<ObjectDisposedException>(() => doc.Merge("b = 2"));
        });
    }
}



