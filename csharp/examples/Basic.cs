using AamCsharp;

// Example 1: Basic parsing
Console.WriteLine("=== Example 1: Basic Parsing ===");
var basicContent = @"
# Server configuration
host = localhost
port = 8080
debug = true
";

try
{
    using var doc = AamDocument.Parse(basicContent);
    Console.WriteLine($"Host: {doc.FindObj("host")}");
    Console.WriteLine($"Port: {doc.FindObj("port")}");
    Console.WriteLine($"Debug: {doc.FindObj("debug")}");
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Native library not found: {ex.Message}");
}

// Example 2: Merging configurations
Console.WriteLine("\n=== Example 2: Merging Configurations ===");
var baseConfig = @"
database = postgres
cache = redis
";

var overrides = @"
database = mysql
debug = true
";

try
{
    using var doc = AamDocument.Parse(baseConfig);
    Console.WriteLine("Base configuration:");
    Console.WriteLine($"  Database: {doc.FindObj("database")}");
    Console.WriteLine($"  Cache: {doc.FindObj("cache")}");

    doc.Merge(overrides);
    Console.WriteLine("\nAfter merge:");
    Console.WriteLine($"  Database: {doc.FindObj("database")}");
    Console.WriteLine($"  Cache: {doc.FindObj("cache")}");
    Console.WriteLine($"  Debug: {doc.FindObj("debug")}");
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Native library not found: {ex.Message}");
}

// Example 3: Finding keys by value
Console.WriteLine("\n=== Example 3: Finding Keys by Value ===");
var registryContent = @"
primary_db = postgresql
backup_db = mysql
cache_layer = redis
message_queue = rabbitmq
";

try
{
    using var doc = AamDocument.Parse(registryContent);
    var key = doc.FindKey("redis");
    if (key != null)
    {
        Console.WriteLine($"Found key for 'redis': {key}");
    }
    else
    {
        Console.WriteLine("Key not found");
    }
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Native library not found: {ex.Message}");
}

Console.WriteLine("\nExamples completed!");

