// Package aam provides Go bindings for the aam-rs AAML parser via CGo.
//
// AAML is a line-based key = value configuration format with support for
// directives (@import, @derive, @schema, @type), schema-based type validation,
// and bidirectional / deep reference resolution.
//
// # Prerequisites
//
// The Rust library must be compiled before using this package:
//
//	cargo build --release --features ffi
//
// By default the CGo flags point to ${SRCDIR}/../../target/release (i.e. the
// aam-rs repository root's release output). Override with environment variables
// CGO_CFLAGS and CGO_LDFLAGS if you install the library to a custom location.
//
// # Usage
//
//	doc, err := aam.Parse("host = localhost\nport = 8080\n")
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer doc.Close()
//
//	if val, ok := doc.FindObj("host"); ok {
//	    fmt.Println(val) // "localhost"
//	}
//
// # Memory management
//
// Each [AAML] instance wraps a native AamlHandle allocated on the Rust heap.
// Call [AAML.Close] when done to release it immediately. A runtime finalizer
// is registered as a safety net so the handle is eventually freed even if
// Close is never called.
package aam

/*
#cgo CFLAGS: -I${SRCDIR}/../../include
#cgo linux LDFLAGS: -L${SRCDIR}/../../target/release -laam_rs -ldl -lpthread -lm
#cgo darwin LDFLAGS: -L${SRCDIR}/../../target/release -laam_rs -ldl -lpthread -lm
#cgo windows LDFLAGS: -L${SRCDIR}/../../target/release -laam_rs -lws2_32 -lbcrypt -luserenv
#include "aam.h"
#include <stdlib.h>
*/
import "C"

import (
	"errors"
	"fmt"
	"runtime"
	"unsafe"
)

// AAML wraps a native AamlHandle.
//
// Construct instances with [New], [Parse], or [Load].
// Call [AAML.Close] when done, or use a defer statement.
// The zero value is not usable; always use a constructor.
type AAML struct {
	ptr *C.AamlHandle
}

// newAAML wraps a raw handle and registers a GC finalizer.
func newAAML(ptr *C.AamlHandle) *AAML {
	a := &AAML{ptr: ptr}
	runtime.SetFinalizer(a, (*AAML).Close)
	return a
}

// New creates an empty AAML instance with all default commands registered.
// Returns an error only on allocation failure (extremely rare).
func New() (*AAML, error) {
	ptr := C.aam_new()
	if ptr == nil {
		return nil, errors.New("aam: allocator returned NULL (out of memory)")
	}
	return newAAML(ptr), nil
}

// Parse creates an AAML instance by parsing the given AAML content string.
// Returns an error if the content is syntactically invalid.
func Parse(content string) (*AAML, error) {
	a, err := New()
	if err != nil {
		return nil, err
	}
	cContent := C.CString(content)
	defer C.free(unsafe.Pointer(cContent))
	if rc := C.aam_parse(a.ptr, cContent); rc != 0 {
		msg := a.LastError()
		a.Close()
		return nil, fmt.Errorf("aam: parse: %s", msg)
	}
	return a, nil
}

// Load creates an AAML instance by reading and parsing the `.aam` file at path.
// Returns an error if the file cannot be read or is invalid.
func Load(path string) (*AAML, error) {
	a, err := New()
	if err != nil {
		return nil, err
	}
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))
	if rc := C.aam_load(a.ptr, cPath); rc != 0 {
		msg := a.LastError()
		a.Close()
		return nil, fmt.Errorf("aam: load %q: %s", path, msg)
	}
	return a, nil
}

// Merge parses additional AAML content and merges it into the current instance.
// Keys already present are not overwritten (child-wins semantics).
// Returns an error if the content is invalid or the handle is closed.
func (a *AAML) Merge(content string) error {
	if a.ptr == nil {
		return errors.New("aam: operation on closed handle")
	}
	cContent := C.CString(content)
	defer C.free(unsafe.Pointer(cContent))
	if rc := C.aam_merge(a.ptr, cContent); rc != 0 {
		return fmt.Errorf("aam: merge: %s", a.LastError())
	}
	return nil
}

// FindObj looks up key in the AAML map. It first performs a forward lookup
// (key → value); if that fails it falls back to a reverse lookup
// (search for an entry whose value equals key).
// Returns ("", false) when nothing is found.
func (a *AAML) FindObj(key string) (string, bool) {
	if a.ptr == nil {
		return "", false
	}
	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))
	result := C.aam_find_obj(a.ptr, cKey)
	if result == nil {
		return "", false
	}
	val := C.GoString(result)
	C.aam_string_free(result)
	return val, true
}

// FindKey performs a reverse lookup: it finds the key whose stored value
// equals value.
// Returns ("", false) when no matching key exists.
func (a *AAML) FindKey(value string) (string, bool) {
	if a.ptr == nil {
		return "", false
	}
	cValue := C.CString(value)
	defer C.free(unsafe.Pointer(cValue))
	result := C.aam_find_key(a.ptr, cValue)
	if result == nil {
		return "", false
	}
	key := C.GoString(result)
	C.aam_string_free(result)
	return key, true
}

// FindDeep follows a chain of key → value → key lookups until a terminal
// value is reached or a reference cycle is detected.
// Returns ("", false) when the starting key is not in the map.
func (a *AAML) FindDeep(key string) (string, bool) {
	if a.ptr == nil {
		return "", false
	}
	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))
	result := C.aam_find_deep(a.ptr, cKey)
	if result == nil {
		return "", false
	}
	val := C.GoString(result)
	C.aam_string_free(result)
	return val, true
}

// LastError returns the last error message stored by the most recent
// failed operation on this handle. Returns "" when there is no pending error.
//
// The returned string is only valid until the next API call on this handle.
func (a *AAML) LastError() string {
	if a.ptr == nil {
		return ""
	}
	cErr := C.aam_last_error(a.ptr)
	if cErr == nil {
		return ""
	}
	return C.GoString(cErr)
}

// Close releases the native memory held by this AAML instance.
// It is safe to call Close multiple times; subsequent calls are no-ops.
func (a *AAML) Close() {
	if a.ptr != nil {
		C.aam_free(a.ptr)
		a.ptr = nil
		runtime.SetFinalizer(a, nil)
	}
}

