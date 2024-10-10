package librpc

// Network that can lose requests, lose replies,
// delay messages, and disconnect particular hosts

// Sends libgob-encoded values to ensure that RPCs
// dont' include references to program objects

import (
	"bytes"
	"log"
	"reflect"
	"sync"

	"github.com/Shresth72/raft/pkg/libgob"
)

type reqMsg struct {
	endname  interface{}
	svcMeth  string
	argsType reflect.Type
	args     []byte
	replyCh  chan replyMsg
}

type replyMsg struct {
	ok    bool
	reply []byte
}

type ClientEnd struct {
	endname interface{}
	ch      chan reqMsg
	done    chan struct{}
}

// send a RPC, wait for the reply
func (e *ClientEnd) Call(svcMeth string, args interface{}, reply interface{}) bool {
	req := reqMsg{
		endname:  e.endname,
		svcMeth:  svcMeth,
		argsType: reflect.TypeOf(args),
	}
	req.replyCh = make(chan replyMsg)

	qb := new(bytes.Buffer)
	qe := libgob.NewEncoder(qb)
	qe.Encode(args)
	req.args = qb.Bytes()

	select {
	// Send req to e.ch channel
	case e.ch <- req:

		// Network went down
	case <-e.done:
		return false
	}

	// Wait for reply
	rep := <-req.replyCh
	if rep.ok {
		rb := bytes.NewBuffer(rep.reply)
		rd := libgob.NewDecoder(rb)
		if err := rd.Decode(reply); err != nil {
			log.Fatalf("ClientEnd.Call(): decode reply: %v\n", err)
		}
		return true
	}
	return false
}
