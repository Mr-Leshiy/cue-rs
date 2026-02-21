package main

/*
#include <stdint.h>
#include <stdbool.h>

typedef uintptr_t CueValue;
*/
import "C"

import (
	"sync"

	"unsafe"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
)

// cueCtx is a shared CUE context. Values produced by different contexts cannot
// be mixed, so we use a single global instance for the lifetime of the library.
var cueCtx = cuecontext.New()

var (
	mu sync.Mutex
	// To prevent dealocation, manage the allocated values inside the map
	values = make(map[uintptr]cue.Value)
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
func cue_value_free(handle C.CueValue) {
	mu.Lock()
	delete(values, uintptr(handle))
	mu.Unlock()
}

//export cue_value_validate
func cue_value_validate(handle C.CueValue) C.bool {
	mu.Lock()
	v, ok := values[uintptr(handle)]
	mu.Unlock()

	if !ok {
		return C.bool(false)
	}
	return C.bool(v.Err() == nil)
}

func main() {}
