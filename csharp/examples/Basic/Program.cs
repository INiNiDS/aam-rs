using AamCsharp;

using var doc = AamDocument.Parse("host = localhost\nport = 8080");
Console.WriteLine($"host={doc.FindObj("host")}");


