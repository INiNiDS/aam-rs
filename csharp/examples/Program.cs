using AamCsharp;

var scenario = args.Length > 0 ? args[0].ToLowerInvariant() : "basic";

try
{
    switch (scenario)
    {
        case "basic":
            RunBasic();
            break;
        case "load":
            RunLoad();
            break;
        default:
            Console.WriteLine("Unknown scenario. Use: basic or load");
            Environment.ExitCode = 1;
            break;
    }
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Native library not found: {ex.Message}");
    Console.WriteLine("Build Rust with --features ffi and ensure the runtime library is discoverable.");
    Environment.ExitCode = 2;
}
catch (AamException ex)
{
    Console.WriteLine($"AAML error: {ex.Message}");
    Environment.ExitCode = 3;
}

static void RunBasic()
{
    using var doc = AamDocument.Parse("host = localhost\nport = 8080");
    Console.WriteLine($"host={doc.FindObj("host")}");
    Console.WriteLine($"port={doc.FindObj("port")}");
}

static void RunLoad()
{
    using var doc = AamDocument.Load("config.aam");
    Console.WriteLine($"app_name={doc.FindObj("app_name")}");
    Console.WriteLine($"environment={doc.FindObj("environment")}");
}

