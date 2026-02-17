package main

/*
#include <stdint.h>
*/
import "C"

//export add_u32
func add_u32(a, b C.uint32_t) C.uint32_t {
	return a + b
}

func main() {}
