package aam_test

import (
	"strings"
	"testing"

	"github.com/INiNiDS/aam-rs/go/aam"
)

// ── Construction ─────────────────────────────────────────────────────────────

func TestNew(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatalf("New() unexpected error: %v", err)
	}
	doc.Close()
}

func TestParse_Basic(t *testing.T) {
	doc, err := aam.Parse("host = localhost\nport = 8080\n")
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}
	defer doc.Close()

	assertFindObj(t, doc, "host", "localhost")
	assertFindObj(t, doc, "port", "8080")
}

func TestParse_MultiLine(t *testing.T) {
	content := strings.Join([]string{
		"name = Alice",
		"role = admin",
		"lang = go",
	}, "\n")
	doc, err := aam.Parse(content)
	if err != nil {
		t.Fatalf("Parse() error: %v", err)
	}
	defer doc.Close()

	assertFindObj(t, doc, "name", "Alice")
	assertFindObj(t, doc, "role", "admin")
	assertFindObj(t, doc, "lang", "go")
}

func TestLoad_NonExistentFile(t *testing.T) {
	_, err := aam.Load("/tmp/aam_test_nonexistent_file_abc123.aam")
	if err == nil {
		t.Fatal("Load() of non-existent file: expected error, got nil")
	}
}

// ── FindObj ───────────────────────────────────────────────────────────────────

func TestFindObj_NotFound(t *testing.T) {
	doc, err := aam.Parse("x = 1\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	if _, ok := doc.FindObj("missing_key"); ok {
		t.Error("FindObj(missing_key): want false, got true")
	}
}

func TestFindObj_ClosedHandle(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatal(err)
	}
	doc.Close()

	if _, ok := doc.FindObj("anything"); ok {
		t.Error("FindObj on closed handle: want false, got true")
	}
}

func TestFindObj_ReverseLookupFallback(t *testing.T) {
	doc, err := aam.Parse("username = alice\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	val, ok := doc.FindObj("alice")
	if !ok {
		t.Fatal("FindObj(alice): not found")
	}
	if val != "username" {
		t.Fatalf("FindObj(alice) = %q; want %q", val, "username")
	}
}

// ── FindKey ───────────────────────────────────────────────────────────────────

func TestFindKey_Found(t *testing.T) {
	doc, err := aam.Parse("username = alice\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	key, ok := doc.FindKey("alice")
	if !ok {
		t.Fatal("FindKey(alice): not found")
	}
	if key != "username" {
		t.Fatalf("FindKey(alice) = %q; want %q", key, "username")
	}
}

func TestFindKey_NotFound(t *testing.T) {
	doc, err := aam.Parse("x = 1\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	if _, ok := doc.FindKey("nonexistent_value"); ok {
		t.Error("FindKey(nonexistent_value): want false, got true")
	}
}

// ── FindDeep ─────────────────────────────────────────────────────────────────

func TestFindDeep_Chain(t *testing.T) {
	// a → b → c → hello  (terminal)
	doc, err := aam.Parse("a = b\nb = c\nc = hello\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	val, ok := doc.FindDeep("a")
	if !ok {
		t.Fatal("FindDeep(a): not found")
	}
	if val != "hello" {
		t.Fatalf("FindDeep(a) = %q; want %q", val, "hello")
	}
}

func TestFindDeep_NoCycle_Direct(t *testing.T) {
	doc, err := aam.Parse("key = value\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	val, ok := doc.FindDeep("key")
	if !ok {
		t.Fatal("FindDeep(key): not found")
	}
	if val != "value" {
		t.Fatalf("FindDeep(key) = %q; want %q", val, "value")
	}
}

func TestFindDeep_NotFound(t *testing.T) {
	doc, err := aam.Parse("x = 1\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	if _, ok := doc.FindDeep("missing"); ok {
		t.Error("FindDeep(missing): want false, got true")
	}
}

func TestFindDeep_Cycle(t *testing.T) {
	doc, err := aam.Parse("a = b\nb = c\nc = a\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	val, ok := doc.FindDeep("a")
	if !ok {
		t.Fatal("FindDeep(a): not found")
	}
	if val != "c" {
		t.Fatalf("FindDeep(a) = %q; want %q", val, "c")
	}
}

// ── Merge ────────────────────────────────────────────────────────────────────

func TestMerge_AddsNewKeys(t *testing.T) {
	doc, err := aam.Parse("x = 1\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	if err := doc.Merge("y = 2\n"); err != nil {
		t.Fatalf("Merge() error: %v", err)
	}

	assertFindObj(t, doc, "x", "1")
	assertFindObj(t, doc, "y", "2")
}

func TestMerge_ClosedHandle(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatal(err)
	}
	doc.Close()

	if err := doc.Merge("z = 3\n"); err == nil {
		t.Error("Merge on closed handle: expected error, got nil")
	}
}

func TestMerge_OverwritesExistingKey(t *testing.T) {
	doc, err := aam.Parse("mode = base\n")
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	if err := doc.Merge("mode = override\n"); err != nil {
		t.Fatalf("Merge() error: %v", err)
	}

	assertFindObj(t, doc, "mode", "override")
}

func TestMerge_InvalidContentReturnsError(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatal(err)
	}
	defer doc.Close()

	if err := doc.Merge("invalid_line_without_equals"); err == nil {
		t.Fatal("Merge(invalid): expected error, got nil")
	}
}

// ── Close / finalizer ────────────────────────────────────────────────────────

func TestClose_Idempotent(t *testing.T) {
	doc, err := aam.New()
	if err != nil {
		t.Fatal(err)
	}
	// Close twice — must not panic or double-free.
	doc.Close()
	doc.Close()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

func assertFindObj(t *testing.T, doc *aam.AAML, key, want string) {
	t.Helper()
	got, ok := doc.FindObj(key)
	if !ok {
		t.Errorf("FindObj(%q): not found (want %q)", key, want)
		return
	}
	if got != want {
		t.Errorf("FindObj(%q) = %q; want %q", key, got, want)
	}
}

