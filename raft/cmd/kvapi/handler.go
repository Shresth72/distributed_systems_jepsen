package main

import (
	"fmt"
	"log"
	"net/http"
	"sync"

	"raft/pkg/goraft"
)

type httpServer struct {
	raft *goraft.Server
	db   *sync.Map
}

func (hs httpServer) setHandler(w http.ResponseWriter, r *http.Request) {
	var c command
	c.kind = setCommand
	c.key = r.URL.Query().Get("key")
	c.value = r.URL.Query().Get("value")

	_, err := hs.raft.Apply([][]byte{encodeCommand(c)})
	if err != nil {
		log.Printf("Could not write key-value: %s", err)
		http.Error(w, http.StatusText(http.StatusBadRequest), http.StatusBadRequest)
		return
	}
}

func (hs httpServer) getHandler(w http.ResponseWriter, r *http.Request) {
	var c command
	c.kind = getCommand
	c.key = r.URL.Query().Get("key")

	var value []byte
	var err error
	if r.URL.Query().Get("relaxed") == "true" {
		v, ok := hs.db.Load(c.key)
		if !ok {
			err = fmt.Errorf("Key not found")
		} else {
			value = []byte(v.(string))
		}
	} else {
		var results []goraft.ApplyResult
		results, err = hs.raft.Apply([][]byte{encodeCommand(c)})
		if err == nil {
			if len(results) != 1 {
				err = fmt.Errorf("Expected single response from Raft, got: %d.", len(results))
			} else if results[0].Error != nil {
				err = results[0].Error
			} else {
				value = results[0].Result
			}

		}
	}

	if err != nil {
		log.Printf("Could not encode key-value in http response: %s", err)
		http.Error(w, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
		return
	}

	written := 0
	for written < len(value) {
		n, err := w.Write(value[written:])
		if err != nil {
			log.Printf("Could not encode key-value in http response: %s", err)
			http.Error(w, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
			return
		}

		written += n
	}
}
