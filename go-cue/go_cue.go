package main

/*
#include <stdint.h>

typedef uintptr_t CueValueAddr;
*/
import "C"

import (
	"sync"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	cueyaml "cuelang.org/go/encoding/yaml"
)

type CueContext struct {
	mu sync.Mutex
	// cueCtx is a shared CUE context. Values produced by different contexts cannot
	// be mixed, so we use a single global instance for the lifetime of the library.
	ctx    *cue.Context
	values map[uintptr]cue.Value
	nextID uintptr
}

func (ctx *CueContext) new_value(v cue.Value) C.CueValueAddr {
	ctx.mu.Lock()
	ctx.nextID++
	addr := ctx.nextID
	ctx.values[addr] = v
	ctx.mu.Unlock()
	return C.CueValueAddr(addr)
}

func (ctx *CueContext) get_value(addr C.CueValueAddr) *cue.Value {
	ctx.mu.Lock()
	v, ok := ctx.values[uintptr(addr)]
	ctx.mu.Unlock()
	if !ok {
		return nil
	}
	return &v
}

func (ctx *CueContext) remove_value(addr C.CueValueAddr) {
	ctx.mu.Lock()
	delete(ctx.values, uintptr(addr))
	ctx.mu.Unlock()
}

var cueCtx = CueContext{
	ctx:    cuecontext.New(),
	values: make(map[uintptr]cue.Value),
	nextID: 1,
}

//export cue_value_new
func cue_value_new(input *C.char) C.CueValueAddr {
	s := C.GoString(input)
	v := cueCtx.ctx.CompileString(s)
	return cueCtx.new_value(v)
}

//export cue_value_free
func cue_value_free(addr C.CueValueAddr) {
	cueCtx.remove_value(addr)
}

// <https://pkg.go.dev/cuelang.org/go/cue#Value.Validate>
//
//export cue_value_validate
func cue_value_validate(addr C.CueValueAddr) *C.char {
	v := cueCtx.get_value(addr)
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
func cue_value_to_json(addr C.CueValueAddr) *C.char {
	v := cueCtx.get_value(addr)
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
func cue_value_to_yaml(addr C.CueValueAddr) *C.char {
	v := cueCtx.get_value(addr)
	if v == nil {
		return nil
	}
	data, err := cueyaml.Encode(*v)
	if err != nil {
		return nil
	}
	return C.CString(string(data))
}

// <https://pkg.go.dev/cuelang.org/go/cue#Value.Unify>
//
//export cue_value_unify
func cue_value_unify(addr1 C.CueValueAddr, addr2 C.CueValueAddr) C.CueValueAddr {
	v1 := cueCtx.get_value(addr1)
	if v1 == nil {
		return addr1
	}
	v2 := cueCtx.get_value(addr2)
	if v2 == nil {
		return addr2
	}
	new_v := v1.Unify(*v2)
	return cueCtx.new_value(new_v)
}

func main() {
}
