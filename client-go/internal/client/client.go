package client

import (
	protobuf "client/codegen/protos/protocol"
	"client/internal/protocol"
	"fmt"
	"net"
	"os"
	"path"

	"google.golang.org/protobuf/proto"
)

var (
	// errors
	ErrClientAlreadyConnected = fmt.Errorf("client already connected")
	ErrUnexepectedMessage     = fmt.Errorf("unexpected message type")
)

type Client struct {
	conn net.Conn

	modules map[string]int
}

func NewClient() Client {
	client := Client{
		conn:    nil,
		modules: make(map[string]int),
	}

	return client
}

func (c *Client) Connect(endpoint string) error {
	if c.conn != nil {
		return ErrClientAlreadyConnected
	}

	conn, err := net.Dial("tcp", endpoint)
	if err != nil {
		return err
	}

	c.conn = conn

	return nil
}

func (c *Client) Close() {
	if c.conn != nil {
		c.conn.Close()
		c.conn = nil
	}
}

func (c *Client) RecvMessage() (*protobuf.Protocol, error) {
	envelope := protocol.Envelope{}

	if err := envelope.Read(c.conn); err != nil {
		fmt.Printf("Error during envelope.Read: %s\n", err)
		return nil, err
	}

	data := envelope.Data
	fmt.Printf("envelope.Data: %v\n", data)
	msg := &protobuf.Protocol{}

	err := proto.Unmarshal(data, msg)
	if err != nil {
		fmt.Printf("Error during Unmarshal: %s\n", err)
		return nil, err
	}
	fmt.Printf("Unmarshaled value: %v\n", msg)

	return msg, nil
}

func (c *Client) SendMessage(msg *protobuf.Protocol) error {
	data, err := proto.Marshal(msg)
	if err != nil {
		return err
	}

	envelope := protocol.WrapMessage(data)

	_, err = envelope.Write(c.conn)
	if err != nil {
		return err
	}

	return nil
}

func (c *Client) Load(module string) (int, error) {
	modulePath := path.Join("modules", module, "module.dll")
	_, err := os.Stat(modulePath)
	if err != nil {
		return 0, err
	}

	data, err := os.ReadFile(modulePath)
	if err != nil {
		return 0, err
	}

	load := protobuf.Load{
		Data: data,
	}

	msg := protobuf.Protocol{
		Msg: &protobuf.Protocol_Load{Load: &load},
	}

	err = c.SendMessage(&msg)
	if err != nil {
		return 0, err
	}

	responseMsg, err := c.RecvMessage()
	if err != nil {
		return 0, err
	}

	response := responseMsg.GetLoadResponse()
	if response == nil {
		fmt.Printf("Unexpected response in Load: %v | %v\n", responseMsg, responseMsg.GetMsg())
		return 0, ErrUnexepectedMessage
	}

	c.modules[module] = int(response.ModuleId)

	return int(response.ModuleId), nil
}
