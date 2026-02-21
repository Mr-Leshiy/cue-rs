package main

/*
#include <stdint.h>

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

	addr := C.CueValue(uintptr(unsafe.Pointer(&v)))
	set_value(addr, v)
	return addr
}

//export cue_value_free
func cue_value_free(addr C.CueValue) {
	mu.Lock()
	delete(values, uintptr(addr))
	mu.Unlock()
}

// <https://pkg.go.dev/cuelang.org/go/cue#Value.Validate>
//
//export cue_value_validate
func cue_value_validate(addr C.CueValue) *C.char {
	v := get_value(addr)
	if v == nil {
		return C.CString("unknown handle")
	}
	if err := v.Validate(cue.Concrete(true)); err != nil {
		return C.CString(err.Error())
	}
	return C.CString("")
}

// <https://pkg.go.dev/cuelang.org/go/cue#Value.MarshalJSON>
//
//export cue_value_to_json
func cue_value_to_json(addr C.CueValue) *C.char {
	v := get_value(addr)
	if v == nil {
		return nil
	}
	data, err := v.MarshalJSON()
	if err != nil {
		return nil
	}
	return C.CString(string(data))
}

// <https://pkg.go.dev/cuelang.org/go/encoding/yaml#Encode>
//
//export cue_value_to_yaml
func cue_value_to_yaml(addr C.CueValue) *C.char {
	v := get_value(addr)
	if v == nil {
		return nil
	}
	data, err := cueyaml.Encode(*v)
	if err != nil {
		return nil
	}
	return C.CString(string(data))
}

// <https://pkg.go.dev/cuelang.org/go/cue#Value.UnifyAccept>
//
//export cue_value_unify_accept
func cue_value_unify_accept(addr1 C.CueValue, addr2 C.CueValue) C.CueValue {
	v1 := get_value(addr1)
	if v1 == nil {
		return addr1
	}
	v2 := get_value(addr2)
	if v2 == nil {
		return addr2
	}
	new_v := v1.Unify(*v2)

	addr := C.CueValue(uintptr(unsafe.Pointer(&new_v)))
	set_value(addr, new_v)

	return addr
}

func get_value(addr C.CueValue) *cue.Value {
	mu.Lock()
	v, ok := values[uintptr(addr)]
	mu.Unlock()

	if !ok {
		return nil
	}
	return &v
}

func set_value(addr C.CueValue, v cue.Value) {
	mu.Lock()
	values[uintptr(addr)] = v
	mu.Unlock()
}

func main() {}
