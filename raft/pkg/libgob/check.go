package libgob

import (
	"fmt"
	"reflect"
	"unicode"
	"unicode/utf8"
)

func checkValue(value interface{}) {
	checkType(reflect.TypeOf(value))
}

func checkType(t reflect.Type) {
	k := t.Kind()
	mu.Lock()

	if checked == nil {
		checked = map[reflect.Type]bool{}
	}
	if checked[t] {
		mu.Unlock()
		return
	}

	checked[t] = true
	mu.Unlock()

	switch k {
	case reflect.Struct:
		for i := 0; i < t.NumField(); i++ {
			f := t.Field(i)
			rune, _ := utf8.DecodeRuneInString(f.Name)
			if unicode.IsUpper(rune) == false {
				fmt.Printf(
					"libgob error: lower-case field %v of %v in RPC or persist/snapshot will break your Raft\n",
					f.Name,
					t.Name(),
				)

				mu.Lock()
				errorCount += 1
				mu.Unlock()
			}
			checkType(f.Type)
		}
		return

	case reflect.Slice, reflect.Array, reflect.Ptr:
		checkType(t.Elem())
		return

	case reflect.Map:
		checkType(t.Elem())
		checkType(t.Key())
		return

	default:
		return
	}
}

// Warn for Non-Default Values
// So that, GOB won't overwrite the Non-Default Values
func checkDefault(value interface{}) {
	if value == nil {
		return
	}
	checkDefaultWithValue(reflect.ValueOf(value), 1, "")
}

func checkDefaultWithValue(value reflect.Value, depth int, name string) {
	if depth > 3 {
		return
	}

	t := value.Type()
	k := t.Kind()

	switch k {
	case reflect.Struct:
		for i := 0; i < t.NumField(); i++ {
			vv := value.Field(i)
			tname := t.Field(i).Name
			if name != "" {
				tname = name + "." + tname
			}
			checkDefaultWithValue(vv, depth+1, tname)
		}
		return

	case reflect.Ptr:
		if value.IsNil() {
			return
		}
		checkDefaultWithValue(value.Elem(), depth+1, name)
		return

	case reflect.Bool,
		reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64,
		reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64,
		reflect.Uintptr, reflect.Float32, reflect.Float64,
		reflect.String:
		if reflect.DeepEqual(reflect.Zero(t).Interface(), value.Interface()) == false {
			mu.Lock()
			if errorCount < 1 {
				what := name
				if what == "" {
					what = t.Name()
				}

				// this warning typically arises if code re-uses the same RPC reply
				// variable for multiple RPC calls, or if code restores persisted
				// state into variable that already have non-default values.
				fmt.Printf(
					"labgob warning: Decoding into a non-default variable/field %v may not work\n",
					what,
				)
			}
			errorCount += 1
			mu.Unlock()
		}
		return
	}
}
