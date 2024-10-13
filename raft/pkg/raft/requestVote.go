package raft

import "time"

type RequestVoteArgs struct {
	Term         int
	CandidateId  int
	LastLogIndex int
	LastLogTerm  int
}

type RequestVoteReply struct {
	Term        int
	VoteGranted bool
}

func (rf *Raft) RequestVote(args *RequestVoteArgs, reply *RequestVoteReply) {
	rf.mu.Lock()
	defer rf.mu.Unlock()

	if rf.currentTerm == args.Term && rf.votedFor == args.CandidateId {
		reply.VoteGranted, reply.Term = true, rf.currentTerm
		return
	}
	if rf.currentTerm > args.Term || (rf.currentTerm == args.Term && rf.votedFor != -1) {
		reply.Term, reply.VoteGranted = rf.currentTerm, false
		return
	}
	if args.Term > rf.currentTerm {
		rf.currentTerm, rf.votedFor = args.Term, -1
		rf.status = Follower
	}

	reply.Term = args.Term
	if rf.lastLogIndex-1 >= 0 {
		lastLogTerm := rf.log[rf.lastLogIndex-1].Term
		if lastLogTerm > args.LastLogTerm ||
			(lastLogTerm == args.LastLogTerm && rf.lastLogIndex > args.LastLogIndex) {
			reply.VoteGranted = false
			return
		}
	}

	rf.status = Follower
	rf.lastAccessed = time.Now()
	reply.VoteGranted = true
	rf.votedFor = args.CandidateId
	rf.persist()
}

func (rf *Raft) sendRequestVote(server int, args *RequestVoteArgs, reply *RequestVoteReply) bool {
	ok := rf.peers[server].Call("Raft.RequestVote", args, reply)
	return ok
}
