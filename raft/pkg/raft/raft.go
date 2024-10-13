package raft

import (
	"bytes"
	"encoding/gob"
	"math/rand"
	"net/rpc"
	"sync"
	"time"

	"github.com/Shresth72/raft/pkg/libgob"
	"github.com/Shresth72/raft/pkg/librpc"
)

const (
	Follower = iota
	Candidate
	Leader
)

const (
	MinElectionTimeout = 500
	MaxElectionTimeout = 1000
)

func randTimeout() time.Duration {
	randTimeout := MinElectionTimeout + rand.Intn(MaxElectionTimeout-MinElectionTimeout)
	return time.Duration(randTimeout) * time.Millisecond
}

type CommandTerm struct {
	Command interface{}
	Term    int
}

type ApplyMsg struct {
	CommandValid bool
	Command      interface{}
	CommandIndex int
}

type Raft struct {
	mu        sync.Mutex
	peers     []*librpc.ClientEnd
	peersRpc  rpc.Client
	persistor *Persister
	me        int
	dead      int32

	status       int
	applyMsg     chan ApplyMsg
	currentTerm  int
	votedFor     int
	log          []CommandTerm
	commitIndex  int
	lastApplied  int
	lastLogIndex int
	nextIndex    []int
	matchIndex   []int
	lastAccessed time.Time
}

func (rf *Raft) GetState() (int, bool) {
	var term int
	var isleader bool

	rf.mu.Lock()
	defer rf.mu.Unlock()

	isleader = rf.status == Leader
	term = rf.currentTerm
	return term, isleader
}

func (rf *Raft) persist() {
	w := new(bytes.Buffer)

	// e := libgob.NewEncoder(w)
	e := gob.NewEncoder(w)
	e.Encode(rf.currentTerm)
	e.Encode(rf.votedFor)
	e.Encode(rf.log)

	data := w.Bytes()
	rf.persistor.SaveRaftState(data)
}

func (rf *Raft) readPersist(data []byte) {
	if data == nil || len(data) < 1 {
		return
	}

	rf.mu.Lock()
	defer rf.mu.Unlock()

	r := bytes.NewBuffer(data)
	d := gob.NewDecoder(r)
	currentTerm, votedFor := 0, 0

	if d.Decode(&currentTerm) != nil || d.Decode(&votedFor) != nil || d.Decode(&rf.log) != nil {
	} else {
		rf.currentTerm = currentTerm
		rf.votedFor = votedFor
		rf.lastLogIndex = len(rf.log) - 1
	}
}

type AppendEntriesArgs struct {
	Term         int
	LeaderId     int
	PrevLogIndex int
	PrevLogTerm  int
	Entry        []CommandTerm
	LeaderCommit int
}

type AppendEntriesReply struct {
	Term    int
	Success bool
	Xterm   int
	XIndex  int
	XLen    int
}

func (rf *Raft) AppendEntries(args *AppendEntriesArgs, reply *AppendEntriesReply) {
	rf.mu.Lock()
	defer rf.mu.Unlock()

	reply.Xterm = -1
	reply.XIndex = -1
	reply.XLen = len(rf.log)
	reply.Success = false
	reply.Term = rf.currentTerm
	if args.Term < rf.currentTerm {
		return
	}

	if len(rf.log) < args.PrevLogIndex+1 {
		return
	}

	if rf.log[args.PrevLogIndex].Term != args.PrevLogTerm {
		reply.Xterm = rf.log[args.PrevLogIndex].Term
		for i, v := range rf.log {
			if v.Term == reply.Xterm {
				reply.XIndex = i
				break
			}
		}
		return
	}
}
