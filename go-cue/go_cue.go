package main

/*
#include <stdint.h>

// Opaque handle to a compiled CUE value managed on the Go side.
typedef uintptr_t CueValue;
*/
import "C"

import (
	"sync"
	"unsafe"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	cueyaml "cuelang.org/go/encoding/yaml"
)

var (
	mu     sync.Mutex
	values = make(map[uintptr]cue.Value)
	// cueCtx is a shared CUE context. Values produced by different contexts cannot
	// be mixed, so we use a single global instance for the lifetime of the library.
	cueCtx = cuecontext.New()
)

//export cue_value_new
func cue_value_new(input *C.char) C.CueValue {
	s := C.GoString(input)
	v := cueCtx.CompileString(s)

	mu.Lock()
	addr := uintptr(unsafe.Pointer(&v))
	values[addr] = v
	mu.Unlock()

	return C.CueValue(addr)
}

//export cue_value_free
func cue_value_free(addr C.CueValue) {
	mu.Lock()
	delete(values, uintptr(addr))
	mu.Unlock()
}

// cue_value_validate returns the error message for the given CueValue as a
// null-terminated C string allocated with malloc. An empty string means the
// value is valid. The caller is responsible for freeing the returned pointer.
//
//export cue_value_validate
func cue_value_validate(handle C.CueValue) *C.char {
	mu.Lock()
	v, ok := values[uintptr(handle)]
	mu.Unlock()

	if !ok {
		return C.CString("unknown handle")
	}
	if err := v.Validate(); err != nil {
		return C.CString(err.Error())
	}
	return C.CString("")
}

// cue_value_to_json encodes the CueValue as a JSON string allocated with malloc.
// Returns NULL on error (unknown handle or encoding failure).
// The caller is responsible for freeing the returned pointer.
//
//export cue_value_to_json
func cue_value_to_json(addr C.CueValue) *C.char {
	mu.Lock()
	v, ok := values[uintptr(addr)]
	mu.Unlock()

	if !ok {
		return nil
	}
	data, err := v.MarshalJSON()
	if err != nil {
		return nil
	}
	return C.CString(string(data))
}

// cue_value_to_yaml encodes the CueValue as a YAML string allocated with malloc.
// Returns NULL on error (unknown handle or encoding failure).
// The caller is responsible for freeing the returned pointer.
//
//export cue_value_to_yaml
func cue_value_to_yaml(handle C.CueValue) *C.char {
	mu.Lock()
	v, ok := values[uintptr(handle)]
	mu.Unlock()

	if !ok {
		return nil
	}
	data, err := cueyaml.Encode(v)
	if err != nil {
		return nil
	}
	return C.CString(string(data))
}

func main() {}
