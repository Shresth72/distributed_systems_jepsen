package libgob

import (
	"encoding/gob"
	"io"
	"reflect"
	"sync"
)

// Warns about non-capitalized field names

var (
	mu         sync.Mutex
	errorCount int
	checked    map[reflect.Type]bool
)

// Encoder
type LibEncoder struct {
	gob *gob.Encoder
}

func NewEncoder(w io.Writer) *LibEncoder {
	enc := &LibEncoder{}
	enc.gob = gob.NewEncoder(w)
	return enc
}

func (enc *LibEncoder) Encode(e interface{}) error {
	checkValue(e)
	return enc.gob.Encode(e)
}

func (enc *LibEncoder) EncodeValue(value reflect.Value) error {
	checkValue(value.Interface())
	return enc.gob.EncodeValue(value)
}

// Decoder
type LibDecoder struct {
	gob gob.Decoder
}

func NewDecoder(r io.Reader) *LibDecoder {
	dec := &LibDecoder{}
	dec.gob = *gob.NewDecoder(r)
	return dec
}

func (dec *LibDecoder) Decode(e interface{}) error {
	checkValue(e)
	checkDefault(e)
	return dec.gob.Decode(e)
}

func (dec *LibDecoder) DecodeValue(value reflect.Value) error {
	// checkValue(value.Interface())
	// checkDefault(value.Interface())
	return dec.gob.DecodeValue(value)
}

// Register
func Register(value interface{}) {
	checkValue(value)
	gob.Register(value)
}

func RegisterName(name string, value interface{}) {
	checkValue(value)
	gob.RegisterName(name, value)
}
