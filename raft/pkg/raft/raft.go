package raft

import "sync"

const (
	Follower = iota
	Candidate
	Leader
)

type CommandTerm struct {
	Command interface{}
	Term    int
}

type Raft struct {
	mu sync.Mutex
}
