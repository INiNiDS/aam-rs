// Package aam provides Go bindings for the aam-rs AAM parser via CGo.
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

// AAM wraps a native AamHandle.
type AAM struct {
	ptr *C.AamHandle
}

// Deprecated: Use AAM instead.
type AAML = AAM

func newAAM(ptr *C.AamHandle) *AAM {
	a := &AAM{ptr: ptr}
	runtime.SetFinalizer(a, (*AAM).Close)
	return a
}

// New creates an empty AAM instance.
func New() (*AAM, error) {
	ptr := C.aam_new()
	if ptr == nil {
		return nil, errors.New("aam: allocator returned NULL (out of memory)")
	}
	return newAAM(ptr), nil
}

// Parse creates an AAM instance by parsing the given AAM content string.
func Parse(content string) (*AAM, error) {
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

// Load creates an AAM instance by reading and parsing the `.aam` file at path.
func Load(path string) (*AAM, error) {
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

// Format takes a raw AAM string and returns a standardized formatted version.
func (a *AAM) Format(content string) (string, error) {
	if a.ptr == nil {
		return "", errors.New("aam: operation on closed handle")
	}
	cContent := C.CString(content)
	defer C.free(unsafe.Pointer(cContent))
	
	result := C.aam_format(a.ptr, cContent)
	if result == nil {
		return "", fmt.Errorf("aam: format: %s", a.LastError())
	}
	
	val := C.GoString(result)
	C.aam_string_free(result)
	return val, nil
}

// Merge parses additional AAM content and merges it into the current instance.
func (a *AAM) Merge(content string) error {
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

// FindObj looks up key in the AAM map.
func (a *AAM) FindObj(key string) (string, bool) {
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

// FindKey performs a reverse lookup.
func (a *AAM) FindKey(value string) (string, bool) {
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

// FindDeep follows a chain of key → value → key lookups until a terminal.
func (a *AAM) FindDeep(key string) (string, bool) {
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

// LastError returns the last error message stored.
func (a *AAM) LastError() string {
	if a.ptr == nil {
		return ""
	}
	cErr := C.aam_last_error(a.ptr)
	if cErr == nil {
		return ""
	}
	return C.GoString(cErr)
}

// Close releases the native memory held by this AAM instance.
func (a *AAM) Close() {
	if a.ptr != nil {
		C.aam_free(a.ptr)
		a.ptr = nil
		runtime.SetFinalizer(a, nil)
	}
}
