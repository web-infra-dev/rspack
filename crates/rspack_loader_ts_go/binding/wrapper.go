package main

import (
	"C"

	"github.com/microsoft/typescript-go/pkg/api"
)

//export TranspileModule
func TranspileModule(source *C.char) *C.char {
	s := C.GoString(source)
	result := api.TranspileModule(s)
	return C.CString(result)
}

func main() {}
