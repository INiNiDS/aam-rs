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
	"strings"
	"unsafe"
)

// AAM wraps a native AamHandle.
type AAM struct {
	ptr *C.AamHandle
}

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

// Get retrieves a string value by its key.
func (a *AAM) Get(key string) (string, bool) {
	if a.ptr == nil {
		return "", false
	}
	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))

	result := C.aam_get(a.ptr, cKey)
	if result == nil {
		return "", false
	}
	val := C.GoString(result)
	C.aam_string_free(result)
	return val, true
}


// ReverseSearch finds all keys that match the specified target value.
func (a *AAM) ReverseSearch(value string) []string {
	if a.ptr == nil {
		return nil
	}
	cValue := C.CString(value)
	defer C.free(unsafe.Pointer(cValue))
	return parseCList(C.aam_reverse_search(a.ptr, cValue))
}

// DeepSearch finds all key-value pairs where the key contains the specified pattern.
func (a *AAM) DeepSearch(pattern string) map[string]string {
	if a.ptr == nil {
		return nil
	}
	cPattern := C.CString(pattern)
	defer C.free(unsafe.Pointer(cPattern))
	return parseCMap(C.aam_deep_search(a.ptr, cPattern))
}

// Find is a smart lookup: tries key exact match (O(1)), then searches values (O(N)).
func (a *AAM) Find(query string) map[string]string {
	if a.ptr == nil {
		return nil
	}
	cQuery := C.CString(query)
	defer C.free(unsafe.Pointer(cQuery))
	return parseCMap(C.aam_find(a.ptr, cQuery))
}

// SchemaNames returns all registered schema names.
func (a *AAM) SchemaNames() []string {
	if a.ptr == nil {
		return nil
	}
	return parseCList(C.aam_schema_names(a.ptr))
}

// TypeNames returns all registered custom type names.
func (a *AAM) TypeNames() []string {
	if a.ptr == nil {
		return nil
	}
	return parseCList(C.aam_type_names(a.ptr))
}

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

func (a *AAM) Close() {
	if a.ptr != nil {
		C.aam_free(a.ptr)
		a.ptr = nil
		runtime.SetFinalizer(a, nil)
	}
}

func parseCList(cStr *C.char) []string {
	if cStr == nil {
		return []string{}
	}
	goStr := C.GoString(cStr)
	C.aam_string_free(cStr)

	if goStr == "" {
		return []string{}
	}
	return strings.Split(goStr, ",")
}

func parseCMap(cStr *C.char) map[string]string {
	res := make(map[string]string)
	if cStr == nil {
		return res
	}
	goStr := C.GoString(cStr)
	C.aam_string_free(cStr)

	if goStr == "" {
		return res
	}

	lines := strings.Split(goStr, "\n")
	for _, line := range lines {
		parts := strings.SplitN(line, "=", 2)
		if len(parts) == 2 {
			res[parts[0]] = parts[1]
		}
	}
	return res
}