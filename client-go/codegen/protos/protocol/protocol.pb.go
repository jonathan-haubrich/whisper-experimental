// Code generated by protoc-gen-go. DO NOT EDIT.
// versions:
// 	protoc-gen-go v1.36.6
// 	protoc        v6.30.2
// source: protocol.proto

package protocol

import (
	protoreflect "google.golang.org/protobuf/reflect/protoreflect"
	protoimpl "google.golang.org/protobuf/runtime/protoimpl"
	reflect "reflect"
	sync "sync"
	unsafe "unsafe"
)

const (
	// Verify that this generated code is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(20 - protoimpl.MinVersion)
	// Verify that runtime/protoimpl is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(protoimpl.MaxVersion - 20)
)

// Mapping from Rust enum Type
type Type int32

const (
	Type_TYPE_INVALID       Type = 0 // Default fallback
	Type_TYPE_LOAD          Type = 1
	Type_TYPE_COMMAND       Type = 2
	Type_TYPE_RESPONSE      Type = 3
	Type_TYPE_LOAD_RESPONSE Type = 4
)

// Enum value maps for Type.
var (
	Type_name = map[int32]string{
		0: "TYPE_INVALID",
		1: "TYPE_LOAD",
		2: "TYPE_COMMAND",
		3: "TYPE_RESPONSE",
		4: "TYPE_LOAD_RESPONSE",
	}
	Type_value = map[string]int32{
		"TYPE_INVALID":       0,
		"TYPE_LOAD":          1,
		"TYPE_COMMAND":       2,
		"TYPE_RESPONSE":      3,
		"TYPE_LOAD_RESPONSE": 4,
	}
)

func (x Type) Enum() *Type {
	p := new(Type)
	*p = x
	return p
}

func (x Type) String() string {
	return protoimpl.X.EnumStringOf(x.Descriptor(), protoreflect.EnumNumber(x))
}

func (Type) Descriptor() protoreflect.EnumDescriptor {
	return file_protocol_proto_enumTypes[0].Descriptor()
}

func (Type) Type() protoreflect.EnumType {
	return &file_protocol_proto_enumTypes[0]
}

func (x Type) Number() protoreflect.EnumNumber {
	return protoreflect.EnumNumber(x)
}

// Deprecated: Use Type.Descriptor instead.
func (Type) EnumDescriptor() ([]byte, []int) {
	return file_protocol_proto_rawDescGZIP(), []int{0}
}

// Common header
type Header struct {
	state         protoimpl.MessageState `protogen:"open.v1"`
	Type          Type                   `protobuf:"varint,1,opt,name=type,proto3,enum=protocol.Type" json:"type,omitempty"`
	Len           uint64                 `protobuf:"varint,2,opt,name=len,proto3" json:"len,omitempty"`
	unknownFields protoimpl.UnknownFields
	sizeCache     protoimpl.SizeCache
}

func (x *Header) Reset() {
	*x = Header{}
	mi := &file_protocol_proto_msgTypes[0]
	ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
	ms.StoreMessageInfo(mi)
}

func (x *Header) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*Header) ProtoMessage() {}

func (x *Header) ProtoReflect() protoreflect.Message {
	mi := &file_protocol_proto_msgTypes[0]
	if x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use Header.ProtoReflect.Descriptor instead.
func (*Header) Descriptor() ([]byte, []int) {
	return file_protocol_proto_rawDescGZIP(), []int{0}
}

func (x *Header) GetType() Type {
	if x != nil {
		return x.Type
	}
	return Type_TYPE_INVALID
}

func (x *Header) GetLen() uint64 {
	if x != nil {
		return x.Len
	}
	return 0
}

// Represents a load message with variable length bytes
type Load struct {
	state         protoimpl.MessageState `protogen:"open.v1"`
	Data          []byte                 `protobuf:"bytes,1,opt,name=data,proto3" json:"data,omitempty"`
	unknownFields protoimpl.UnknownFields
	sizeCache     protoimpl.SizeCache
}

func (x *Load) Reset() {
	*x = Load{}
	mi := &file_protocol_proto_msgTypes[1]
	ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
	ms.StoreMessageInfo(mi)
}

func (x *Load) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*Load) ProtoMessage() {}

func (x *Load) ProtoReflect() protoreflect.Message {
	mi := &file_protocol_proto_msgTypes[1]
	if x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use Load.ProtoReflect.Descriptor instead.
func (*Load) Descriptor() ([]byte, []int) {
	return file_protocol_proto_rawDescGZIP(), []int{1}
}

func (x *Load) GetData() []byte {
	if x != nil {
		return x.Data
	}
	return nil
}

type LoadResponse struct {
	state         protoimpl.MessageState `protogen:"open.v1"`
	ModuleId      uint32                 `protobuf:"varint,1,opt,name=module_id,json=moduleId,proto3" json:"module_id,omitempty"`
	unknownFields protoimpl.UnknownFields
	sizeCache     protoimpl.SizeCache
}

func (x *LoadResponse) Reset() {
	*x = LoadResponse{}
	mi := &file_protocol_proto_msgTypes[2]
	ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
	ms.StoreMessageInfo(mi)
}

func (x *LoadResponse) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*LoadResponse) ProtoMessage() {}

func (x *LoadResponse) ProtoReflect() protoreflect.Message {
	mi := &file_protocol_proto_msgTypes[2]
	if x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use LoadResponse.ProtoReflect.Descriptor instead.
func (*LoadResponse) Descriptor() ([]byte, []int) {
	return file_protocol_proto_rawDescGZIP(), []int{2}
}

func (x *LoadResponse) GetModuleId() uint32 {
	if x != nil {
		return x.ModuleId
	}
	return 0
}

// A command with a module ID, command ID, and payload
type Command struct {
	state         protoimpl.MessageState `protogen:"open.v1"`
	ModuleId      uint64                 `protobuf:"varint,1,opt,name=module_id,json=moduleId,proto3" json:"module_id,omitempty"`
	Id            uint64                 `protobuf:"varint,2,opt,name=id,proto3" json:"id,omitempty"`
	Data          []byte                 `protobuf:"bytes,3,opt,name=data,proto3" json:"data,omitempty"`
	unknownFields protoimpl.UnknownFields
	sizeCache     protoimpl.SizeCache
}

func (x *Command) Reset() {
	*x = Command{}
	mi := &file_protocol_proto_msgTypes[3]
	ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
	ms.StoreMessageInfo(mi)
}

func (x *Command) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*Command) ProtoMessage() {}

func (x *Command) ProtoReflect() protoreflect.Message {
	mi := &file_protocol_proto_msgTypes[3]
	if x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use Command.ProtoReflect.Descriptor instead.
func (*Command) Descriptor() ([]byte, []int) {
	return file_protocol_proto_rawDescGZIP(), []int{3}
}

func (x *Command) GetModuleId() uint64 {
	if x != nil {
		return x.ModuleId
	}
	return 0
}

func (x *Command) GetId() uint64 {
	if x != nil {
		return x.Id
	}
	return 0
}

func (x *Command) GetData() []byte {
	if x != nil {
		return x.Data
	}
	return nil
}

// A response with data
type Response struct {
	state         protoimpl.MessageState `protogen:"open.v1"`
	Data          []byte                 `protobuf:"bytes,1,opt,name=data,proto3" json:"data,omitempty"`
	unknownFields protoimpl.UnknownFields
	sizeCache     protoimpl.SizeCache
}

func (x *Response) Reset() {
	*x = Response{}
	mi := &file_protocol_proto_msgTypes[4]
	ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
	ms.StoreMessageInfo(mi)
}

func (x *Response) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*Response) ProtoMessage() {}

func (x *Response) ProtoReflect() protoreflect.Message {
	mi := &file_protocol_proto_msgTypes[4]
	if x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use Response.ProtoReflect.Descriptor instead.
func (*Response) Descriptor() ([]byte, []int) {
	return file_protocol_proto_rawDescGZIP(), []int{4}
}

func (x *Response) GetData() []byte {
	if x != nil {
		return x.Data
	}
	return nil
}

// A top-level message that can be one of Load or Command
type Protocol struct {
	state protoimpl.MessageState `protogen:"open.v1"`
	// Types that are valid to be assigned to Msg:
	//
	//	*Protocol_Load
	//	*Protocol_Command
	//	*Protocol_Response
	//	*Protocol_LoadResponse
	Msg           isProtocol_Msg `protobuf_oneof:"msg"`
	unknownFields protoimpl.UnknownFields
	sizeCache     protoimpl.SizeCache
}

func (x *Protocol) Reset() {
	*x = Protocol{}
	mi := &file_protocol_proto_msgTypes[5]
	ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
	ms.StoreMessageInfo(mi)
}

func (x *Protocol) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*Protocol) ProtoMessage() {}

func (x *Protocol) ProtoReflect() protoreflect.Message {
	mi := &file_protocol_proto_msgTypes[5]
	if x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use Protocol.ProtoReflect.Descriptor instead.
func (*Protocol) Descriptor() ([]byte, []int) {
	return file_protocol_proto_rawDescGZIP(), []int{5}
}

func (x *Protocol) GetMsg() isProtocol_Msg {
	if x != nil {
		return x.Msg
	}
	return nil
}

func (x *Protocol) GetLoad() *Load {
	if x != nil {
		if x, ok := x.Msg.(*Protocol_Load); ok {
			return x.Load
		}
	}
	return nil
}

func (x *Protocol) GetCommand() *Command {
	if x != nil {
		if x, ok := x.Msg.(*Protocol_Command); ok {
			return x.Command
		}
	}
	return nil
}

func (x *Protocol) GetResponse() *Response {
	if x != nil {
		if x, ok := x.Msg.(*Protocol_Response); ok {
			return x.Response
		}
	}
	return nil
}

func (x *Protocol) GetLoadResponse() *LoadResponse {
	if x != nil {
		if x, ok := x.Msg.(*Protocol_LoadResponse); ok {
			return x.LoadResponse
		}
	}
	return nil
}

type isProtocol_Msg interface {
	isProtocol_Msg()
}

type Protocol_Load struct {
	Load *Load `protobuf:"bytes,1,opt,name=load,proto3,oneof"`
}

type Protocol_Command struct {
	Command *Command `protobuf:"bytes,2,opt,name=command,proto3,oneof"`
}

type Protocol_Response struct {
	Response *Response `protobuf:"bytes,3,opt,name=response,proto3,oneof"`
}

type Protocol_LoadResponse struct {
	LoadResponse *LoadResponse `protobuf:"bytes,4,opt,name=load_response,json=loadResponse,proto3,oneof"`
}

func (*Protocol_Load) isProtocol_Msg() {}

func (*Protocol_Command) isProtocol_Msg() {}

func (*Protocol_Response) isProtocol_Msg() {}

func (*Protocol_LoadResponse) isProtocol_Msg() {}

var File_protocol_proto protoreflect.FileDescriptor

const file_protocol_proto_rawDesc = "" +
	"\n" +
	"\x0eprotocol.proto\x12\bprotocol\">\n" +
	"\x06Header\x12\"\n" +
	"\x04type\x18\x01 \x01(\x0e2\x0e.protocol.TypeR\x04type\x12\x10\n" +
	"\x03len\x18\x02 \x01(\x04R\x03len\"\x1a\n" +
	"\x04Load\x12\x12\n" +
	"\x04data\x18\x01 \x01(\fR\x04data\"+\n" +
	"\fLoadResponse\x12\x1b\n" +
	"\tmodule_id\x18\x01 \x01(\rR\bmoduleId\"J\n" +
	"\aCommand\x12\x1b\n" +
	"\tmodule_id\x18\x01 \x01(\x04R\bmoduleId\x12\x0e\n" +
	"\x02id\x18\x02 \x01(\x04R\x02id\x12\x12\n" +
	"\x04data\x18\x03 \x01(\fR\x04data\"\x1e\n" +
	"\bResponse\x12\x12\n" +
	"\x04data\x18\x01 \x01(\fR\x04data\"\xd7\x01\n" +
	"\bProtocol\x12$\n" +
	"\x04load\x18\x01 \x01(\v2\x0e.protocol.LoadH\x00R\x04load\x12-\n" +
	"\acommand\x18\x02 \x01(\v2\x11.protocol.CommandH\x00R\acommand\x120\n" +
	"\bresponse\x18\x03 \x01(\v2\x12.protocol.ResponseH\x00R\bresponse\x12=\n" +
	"\rload_response\x18\x04 \x01(\v2\x16.protocol.LoadResponseH\x00R\floadResponseB\x05\n" +
	"\x03msg*d\n" +
	"\x04Type\x12\x10\n" +
	"\fTYPE_INVALID\x10\x00\x12\r\n" +
	"\tTYPE_LOAD\x10\x01\x12\x10\n" +
	"\fTYPE_COMMAND\x10\x02\x12\x11\n" +
	"\rTYPE_RESPONSE\x10\x03\x12\x16\n" +
	"\x12TYPE_LOAD_RESPONSE\x10\x04B\fZ\n" +
	"./protocolb\x06proto3"

var (
	file_protocol_proto_rawDescOnce sync.Once
	file_protocol_proto_rawDescData []byte
)

func file_protocol_proto_rawDescGZIP() []byte {
	file_protocol_proto_rawDescOnce.Do(func() {
		file_protocol_proto_rawDescData = protoimpl.X.CompressGZIP(unsafe.Slice(unsafe.StringData(file_protocol_proto_rawDesc), len(file_protocol_proto_rawDesc)))
	})
	return file_protocol_proto_rawDescData
}

var file_protocol_proto_enumTypes = make([]protoimpl.EnumInfo, 1)
var file_protocol_proto_msgTypes = make([]protoimpl.MessageInfo, 6)
var file_protocol_proto_goTypes = []any{
	(Type)(0),            // 0: protocol.Type
	(*Header)(nil),       // 1: protocol.Header
	(*Load)(nil),         // 2: protocol.Load
	(*LoadResponse)(nil), // 3: protocol.LoadResponse
	(*Command)(nil),      // 4: protocol.Command
	(*Response)(nil),     // 5: protocol.Response
	(*Protocol)(nil),     // 6: protocol.Protocol
}
var file_protocol_proto_depIdxs = []int32{
	0, // 0: protocol.Header.type:type_name -> protocol.Type
	2, // 1: protocol.Protocol.load:type_name -> protocol.Load
	4, // 2: protocol.Protocol.command:type_name -> protocol.Command
	5, // 3: protocol.Protocol.response:type_name -> protocol.Response
	3, // 4: protocol.Protocol.load_response:type_name -> protocol.LoadResponse
	5, // [5:5] is the sub-list for method output_type
	5, // [5:5] is the sub-list for method input_type
	5, // [5:5] is the sub-list for extension type_name
	5, // [5:5] is the sub-list for extension extendee
	0, // [0:5] is the sub-list for field type_name
}

func init() { file_protocol_proto_init() }
func file_protocol_proto_init() {
	if File_protocol_proto != nil {
		return
	}
	file_protocol_proto_msgTypes[5].OneofWrappers = []any{
		(*Protocol_Load)(nil),
		(*Protocol_Command)(nil),
		(*Protocol_Response)(nil),
		(*Protocol_LoadResponse)(nil),
	}
	type x struct{}
	out := protoimpl.TypeBuilder{
		File: protoimpl.DescBuilder{
			GoPackagePath: reflect.TypeOf(x{}).PkgPath(),
			RawDescriptor: unsafe.Slice(unsafe.StringData(file_protocol_proto_rawDesc), len(file_protocol_proto_rawDesc)),
			NumEnums:      1,
			NumMessages:   6,
			NumExtensions: 0,
			NumServices:   0,
		},
		GoTypes:           file_protocol_proto_goTypes,
		DependencyIndexes: file_protocol_proto_depIdxs,
		EnumInfos:         file_protocol_proto_enumTypes,
		MessageInfos:      file_protocol_proto_msgTypes,
	}.Build()
	File_protocol_proto = out.File
	file_protocol_proto_goTypes = nil
	file_protocol_proto_depIdxs = nil
}
