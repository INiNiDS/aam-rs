using AamCsharp;

Console.WriteLine("=== C# AAML Configuration Example ===\n");

try
{
    // Load configuration from file
    Console.WriteLine("Loading configuration from config.aam...");
    using var config = AamDocument.Load("config.aam");

    Console.WriteLine("\n--- Application Info ---");
    Console.WriteLine($"Application: {config.FindObj("app_name")} v{config.FindObj("app_version")}");
    Console.WriteLine($"Environment: {config.FindObj("environment")}");

    Console.WriteLine("\n--- Server Configuration ---");
    Console.WriteLine($"Host: {config.FindObj("server_host")}");
    Console.WriteLine($"Port: {config.FindObj("server_port")}");
    Console.WriteLine($"Timeout: {config.FindObj("server_timeout")}ms");

    Console.WriteLine("\n--- Database Configuration ---");
    Console.WriteLine($"Type: {config.FindObj("db_type")}");
    Console.WriteLine($"Host: {config.FindObj("db_host")}:{config.FindObj("db_port")}");
    Console.WriteLine($"Database: {config.FindObj("db_name")}");
    Console.WriteLine($"Max Connections: {config.FindObj("db_max_connections")}");

    Console.WriteLine("\n--- Logging Settings ---");
    Console.WriteLine($"Level: {config.FindObj("log_level")}");
    Console.WriteLine($"Format: {config.FindObj("log_format")}");
    Console.WriteLine($"Output: {config.FindObj("log_output")}");

    Console.WriteLine("\n--- Feature Flags ---");
    Console.WriteLine($"Analytics: {config.FindObj("feature_analytics")}");
    Console.WriteLine($"Caching: {config.FindObj("feature_caching")}");
    Console.WriteLine($"Debug Mode: {config.FindObj("feature_debug_mode")}");

    // Dynamic merging example
    Console.WriteLine("\n--- Runtime Configuration Override ---");
    var overrides = @"
server_port = 9090
environment = staging
feature_debug_mode = true
";

    config.Merge(overrides);

    Console.WriteLine($"Port after override: {config.FindObj("server_port")}");
    Console.WriteLine($"Environment after override: {config.FindObj("environment")}");
    Console.WriteLine($"Debug Mode after override: {config.FindObj("feature_debug_mode")}");
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Error: Native library not found - {ex.Message}");
    Console.WriteLine("Please ensure the aam_rs native library is available in your PATH.");
}
catch (InvalidOperationException ex)
{
    Console.WriteLine($"Error parsing configuration: {ex.Message}");
}
catch (FileNotFoundException ex)
{
    Console.WriteLine($"Error: Configuration file not found - {ex.Message}");
}

Console.WriteLine("\nExample completed!");

