package protocol

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
)

var ErrInvalidData = fmt.Errorf("could not unmarshal, data invalid")

var endianness = binary.BigEndian

type Envelope struct {
	Len  uint64
	Data []byte
}

func WrapMessage(data []byte) Envelope {
	return Envelope{
		Len:  uint64(len(data)),
		Data: data,
	}
}

func UnwrapMessage(data []byte) ([]byte, error) {
	envelope := Envelope{}

	err := envelope.UnmarshalBinary(data)
	if err != nil {
		return nil, err
	}

	return envelope.Data, nil
}

func (e *Envelope) UnmarshalBinary(data []byte) error {
	reader := bytes.NewReader(data)

	return e.Read(reader)
}

func (e *Envelope) Read(reader io.Reader) error {
	if err := binary.Read(reader, endianness, &e.Len); err != nil {
		return err
	}

	e.Data = make([]byte, e.Len)
	totalRead := 0
	for totalRead < len(e.Data) {
		bytesRead, err := reader.Read(e.Data[totalRead:])
		if err != nil {
			return err
		}

		totalRead += bytesRead
	}

	fmt.Printf("totalRead: %d\nenvelope: %v\n", totalRead, e)

	return nil
}

func (e Envelope) MarshalBinary() ([]byte, error) {
	data := make([]byte, 0)
	buf := bytes.NewBuffer(data)

	_, err := e.Write(buf)
	if err != nil {
		return nil, err
	}

	return buf.Bytes(), nil
}

func (e Envelope) Write(writer io.Writer) (int, error) {
	fmt.Printf("Writing %d bytes: %d\n", binary.Size(e.Len), e.Len)
	err := binary.Write(writer, endianness, e.Len)
	if err != nil {
		return 0, err
	}

	totalWritten := 0
	for totalWritten < len(e.Data) {
		written, err := writer.Write(e.Data[totalWritten:])
		if err != nil {
			return 0, err
		}

		totalWritten += written
	}

	fmt.Printf("Wrote %d bytes\n", totalWritten)

	return binary.Size(e.Len) + len(e.Data), nil
}
