package main

/*
#include <stdint.h>

// Opaque handle to a compiled CUE value managed on the Go side.
typedef uintptr_t CueValue;
*/
import "C"

import (
	"sync"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
)

// cueCtx is a shared CUE context. Values produced by different contexts cannot
// be mixed, so we use a single global instance for the lifetime of the library.
var cueCtx = cuecontext.New()

var (
	mu     sync.Mutex
	values = make(map[uintptr]cue.Value)
	nextID uintptr = 1
)

//export cue_value_new
func cue_value_new(input *C.char) C.CueValue {
	s := C.GoString(input)
	v := cueCtx.CompileString(s)

	mu.Lock()
	id := nextID
	nextID++
	values[id] = v
	mu.Unlock()

	return C.CueValue(id)
}

//export cue_value_free
func cue_value_free(handle C.CueValue) {
	mu.Lock()
	delete(values, uintptr(handle))
	mu.Unlock()
}

//export cue_value_validate
// cue_value_validate returns the error message for the given CueValue as a
// null-terminated C string allocated with malloc. An empty string means the
// value is valid. The caller is responsible for freeing the returned pointer.
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

func main() {}
