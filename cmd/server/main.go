package main

// go build -o bin/server cmd/server/main.go && ./bin/server

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"sync"

	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"

	echopb "github.com/Shresth72/go_gRPC_tester/proto/echo"
	initpb "github.com/Shresth72/go_gRPC_tester/proto/init"
	uniqueidpb "github.com/Shresth72/go_gRPC_tester/proto/unique_ids"
	broadcastpb "github.com/Shresth72/go_gRPC_tester/proto/broadcast"
)

type server struct {
	initpb.UnimplementedInitServiceServer
	echopb.UnimplementedEchoServiceServer
  uniqueidpb.UnimplementedUniqueIdsServiceServer
  broadcastpb.UnimplementedBroadcastServiceServer

  binaryName  string
	rustProcess *exec.Cmd
	stdinPipe   *os.File
  stdoutPipe  *bufio.Reader
	mu          sync.Mutex
}

func (s *server) captureOutput() {
  scanner := bufio.NewScanner(s.stdoutPipe)
  for scanner.Scan() {
      log.Printf("binary output: %s", scanner.Text())
  }
  if err := scanner.Err(); err != nil {
      log.Printf("error reading from binary output: %v", err)
  }
}

func (s *server) writeToStdin(in interface{}) error {
  s.mu.Lock()
  defer s.mu.Unlock()

  message, err := json.Marshal(in)
  if err != nil {
    return fmt.Errorf("failed to marshal message: %w", err)
  }

  _, err = s.stdinPipe.WriteString(string(message) + "\n")
  if err != nil {
    return fmt.Errorf("failed to write to stdin: %w", err)
  }
  println("writing: ", in, string(message))

  return nil
}

func (s *server) handleBroadcast(request interface{}) (interface{}, error) {
  if err := s.writeToStdin(request); err != nil {
    return nil, err
  }

  switch req := request.(type) {
  case *broadcastpb.BroadcastRequest:
    return &broadcastpb.BroadcastResponse {
      Src: req.Dest,
      Dest: req.Src,
      Body: &broadcastpb.BroadcastResponseBody{
        Type: "broadcast_ok",
      },
    }, nil
  case *broadcastpb.ReadRequest:
    return &broadcastpb.ReadResponse {
      Src: req.Dest,
      Dest: req.Src,
      Body: &broadcastpb.ReadResponseBody {
        Type: "read_ok",
        Messages: []int32{1,2,3},
      },
    }, nil 
  case *broadcastpb.TopologyRequest:
    return &broadcastpb.TopologyResponse {
      Src: req.Dest,
      Dest: req.Src,
      Body: &broadcastpb.TopologyResponseBody {
        Type: "topology_ok",
      },
    }, nil
  default:
    return nil, fmt.Errorf("unsupported request type")
  }
}

func (s *server) SendInit(ctx context.Context, in *initpb.InitRequest) (*initpb.InitResponse, error) {
  if err := s.writeToStdin(in); err != nil {
    return nil, err
  }

  return &initpb.InitResponse {
    Src: in.Dest,
    Dest: in.Src,
    Body: &initpb.InitResponseBody {
      Type: "init_ok",
      MsgId: 0,
      InReplyTo: nil,
    },  
  }, nil
}

func (s *server) SendEcho(ctx context.Context, in *echopb.EchoRequest) (*echopb.EchoResponse, error) {
  if err := s.writeToStdin(in); err != nil {
    return nil, err
  }

  return &echopb.EchoResponse {
    Src: in.Dest,
    Dest: in.Src,
    Body: &echopb.EchoResponseBody {
      Type: "echo_ok",
      Echo: in.Body.Echo,
      MsgId: in.Body.MsgId,
      InReplyTo: in.Body.MsgId,
    },
  }, nil
}

func (s* server) SendUniqueIds(ctx context.Context, in *uniqueidpb.UniqueIdsRequest) (*uniqueidpb.UniqueIdsResponse, error) {
  if err := s.writeToStdin(in); err != nil {
    return nil, err
  }

  return &uniqueidpb.UniqueIdsResponse{
    Src: in.Dest,
    Dest: in.Src,
    Body: &uniqueidpb.UniqueIdsResponseBody{
      Type: "generate_ok",
      Id: 1,
    },
  }, nil
}

func (s* server) SetBinaryName(ctx context.Context, in *initpb.SetBinaryNameRequest) (*initpb.SetBinaryNameResponse, error) {
  s.mu.Lock()
  defer s.mu.Unlock()

  s.binaryName = in.BinaryName
  binaryPath := fmt.Sprintf("/home/shrestha/rust/distributed_systems/target/debug/%s", s.binaryName)
  rustCmd := exec.Command(binaryPath)

  stdinPipe, err := rustCmd.StdinPipe()
  if err != nil {
      return nil, fmt.Errorf("failed to get stdin pipe: %v", err)
  }

  stdoutPipe, err := rustCmd.StdoutPipe()
  if err != nil {
      return nil, fmt.Errorf("failed to get stdout pipe: %v", err)
  }

  rustCmd.Stderr = os.Stderr

  if err := rustCmd.Start(); err != nil {
      return nil, fmt.Errorf("failed to start binary: %v", err)
  }

  s.rustProcess = rustCmd
  s.stdinPipe = stdinPipe.(*os.File)
  s.stdoutPipe = bufio.NewReader(stdoutPipe)

  go s.captureOutput() // Ensure output is captured

  return &initpb.SetBinaryNameResponse{}, nil
}

func (s *server) SendBroadcast(ctx context.Context, in *broadcastpb.BroadcastRequest) (*broadcastpb.BroadcastResponse, error) {
  resp, err := s.handleBroadcast(in)
  if err != nil {
    return nil, err
  }
  return resp.(*broadcastpb.BroadcastResponse), nil
}

func (s *server) SendRead(ctx context.Context, in *broadcastpb.ReadRequest) (*broadcastpb.ReadResponse, error) {
  resp, err := s.handleBroadcast(in)
  if err != nil {
    return nil, err
  }
  return resp.(*broadcastpb.ReadResponse), nil
}

func (s *server) SendTopology(ctx context.Context, in *broadcastpb.TopologyRequest) (*broadcastpb.TopologyResponse, error) {
  resp, err := s.handleBroadcast(in)
  if err != nil {
    return nil, err
  }
  return resp.(*broadcastpb.TopologyResponse), nil
}

func main() {
  s := &server{}

	grpcServer := grpc.NewServer()
	initpb.RegisterInitServiceServer(grpcServer, s)
	echopb.RegisterEchoServiceServer(grpcServer, s)
  uniqueidpb.RegisterUniqueIdsServiceServer(grpcServer, s)
  broadcastpb.RegisterBroadcastServiceServer(grpcServer, s)

	reflection.Register(grpcServer)

	lis, err := net.Listen("tcp", ":5051")
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	log.Printf("Server is listening on :5051")
	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}
