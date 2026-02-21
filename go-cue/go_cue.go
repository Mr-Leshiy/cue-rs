package main

/*
#include <stdint.h>
#include <stdbool.h>
*/
import "C"

import (
	"cuelang.org/go/cue/cuecontext"
)

//export validate
func validate(input *C.char) C.bool {
	s := C.GoString(input)
	ctx := cuecontext.New()
	v := ctx.CompileString(s)
	return C.bool(v.Err() == nil)
}

func main() {}
