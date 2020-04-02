# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: bcdb.proto

from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from google.protobuf import reflection as _reflection
from google.protobuf import symbol_database as _symbol_database
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor.FileDescriptor(
  name='bcdb.proto',
  package='bcdb',
  syntax='proto3',
  serialized_options=None,
  serialized_pb=b'\n\nbcdb.proto\x12\x04\x62\x63\x64\x62\"e\n\x03Tag\x12\x0b\n\x03key\x18\x01 \x01(\t\x12\x10\n\x06string\x18\x02 \x01(\tH\x00\x12\x10\n\x06\x64ouble\x18\x03 \x01(\x01H\x00\x12\x10\n\x06number\x18\x04 \x01(\x03H\x00\x12\x12\n\x08unsigned\x18\x05 \x01(\x04H\x00\x42\x07\n\x05value\"#\n\x08Metadata\x12\x17\n\x04tags\x18\x01 \x03(\x0b\x32\t.bcdb.Tag\"<\n\nSetRequest\x12 \n\x08metadata\x18\x01 \x01(\x0b\x32\x0e.bcdb.Metadata\x12\x0c\n\x04\x64\x61ta\x18\x02 \x01(\x0c\"\x19\n\x0bSetResponse\x12\n\n\x02id\x18\x01 \x01(\t\"\x18\n\nGetRequest\x12\n\n\x02id\x18\x01 \x01(\t\"=\n\x0bGetResponse\x12 \n\x08metadata\x18\x01 \x01(\x0b\x32\x0e.bcdb.Metadata\x12\x0c\n\x04\x64\x61ta\x18\x02 \x01(\x0c\"=\n\rUpdateRequest\x12\n\n\x02id\x18\x01 \x01(\t\x12 \n\x08metadata\x18\x02 \x01(\x0b\x32\x0e.bcdb.Metadata\"\x10\n\x0eUpdateResponse\"\x0e\n\x0cQueryRequest\"\x1a\n\x0cListResponse\x12\n\n\x02id\x18\x01 \x01(\t\"J\n\x0c\x46indResponse\x12\n\n\x02id\x18\x01 \x01(\t\x12 \n\x08metadata\x18\x02 \x01(\x0b\x32\x0e.bcdb.Metadata\x12\x0c\n\x04\x64\x61ta\x18\x03 \x01(\x0c\x32\x81\x02\n\x04\x42\x43\x44\x42\x12,\n\x03Set\x12\x10.bcdb.SetRequest\x1a\x11.bcdb.SetResponse\"\x00\x12,\n\x03Get\x12\x10.bcdb.GetRequest\x1a\x11.bcdb.GetResponse\"\x00\x12\x35\n\x06Update\x12\x13.bcdb.UpdateRequest\x1a\x14.bcdb.UpdateResponse\"\x00\x12\x32\n\x04List\x12\x12.bcdb.QueryRequest\x1a\x12.bcdb.ListResponse\"\x00\x30\x01\x12\x32\n\x04\x46ind\x12\x12.bcdb.QueryRequest\x1a\x12.bcdb.FindResponse\"\x00\x30\x01\x62\x06proto3'
)




_TAG = _descriptor.Descriptor(
  name='Tag',
  full_name='bcdb.Tag',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
    _descriptor.FieldDescriptor(
      name='key', full_name='bcdb.Tag.key', index=0,
      number=1, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
    _descriptor.FieldDescriptor(
      name='string', full_name='bcdb.Tag.string', index=1,
      number=2, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
    _descriptor.FieldDescriptor(
      name='double', full_name='bcdb.Tag.double', index=2,
      number=3, type=1, cpp_type=5, label=1,
      has_default_value=False, default_value=float(0),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
    _descriptor.FieldDescriptor(
      name='number', full_name='bcdb.Tag.number', index=3,
      number=4, type=3, cpp_type=2, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
    _descriptor.FieldDescriptor(
      name='unsigned', full_name='bcdb.Tag.unsigned', index=4,
      number=5, type=4, cpp_type=4, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
    _descriptor.OneofDescriptor(
      name='value', full_name='bcdb.Tag.value',
      index=0, containing_type=None, fields=[]),
  ],
  serialized_start=20,
  serialized_end=121,
)


_METADATA = _descriptor.Descriptor(
  name='Metadata',
  full_name='bcdb.Metadata',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
    _descriptor.FieldDescriptor(
      name='tags', full_name='bcdb.Metadata.tags', index=0,
      number=1, type=11, cpp_type=10, label=3,
      has_default_value=False, default_value=[],
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=123,
  serialized_end=158,
)


_SETREQUEST = _descriptor.Descriptor(
  name='SetRequest',
  full_name='bcdb.SetRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
    _descriptor.FieldDescriptor(
      name='metadata', full_name='bcdb.SetRequest.metadata', index=0,
      number=1, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
    _descriptor.FieldDescriptor(
      name='data', full_name='bcdb.SetRequest.data', index=1,
      number=2, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=160,
  serialized_end=220,
)


_SETRESPONSE = _descriptor.Descriptor(
  name='SetResponse',
  full_name='bcdb.SetResponse',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
    _descriptor.FieldDescriptor(
      name='id', full_name='bcdb.SetResponse.id', index=0,
      number=1, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=222,
  serialized_end=247,
)


_GETREQUEST = _descriptor.Descriptor(
  name='GetRequest',
  full_name='bcdb.GetRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
    _descriptor.FieldDescriptor(
      name='id', full_name='bcdb.GetRequest.id', index=0,
      number=1, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=249,
  serialized_end=273,
)


_GETRESPONSE = _descriptor.Descriptor(
  name='GetResponse',
  full_name='bcdb.GetResponse',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
    _descriptor.FieldDescriptor(
      name='metadata', full_name='bcdb.GetResponse.metadata', index=0,
      number=1, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
    _descriptor.FieldDescriptor(
      name='data', full_name='bcdb.GetResponse.data', index=1,
      number=2, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=275,
  serialized_end=336,
)


_UPDATEREQUEST = _descriptor.Descriptor(
  name='UpdateRequest',
  full_name='bcdb.UpdateRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
    _descriptor.FieldDescriptor(
      name='id', full_name='bcdb.UpdateRequest.id', index=0,
      number=1, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
    _descriptor.FieldDescriptor(
      name='metadata', full_name='bcdb.UpdateRequest.metadata', index=1,
      number=2, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=338,
  serialized_end=399,
)


_UPDATERESPONSE = _descriptor.Descriptor(
  name='UpdateResponse',
  full_name='bcdb.UpdateResponse',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=401,
  serialized_end=417,
)


_QUERYREQUEST = _descriptor.Descriptor(
  name='QueryRequest',
  full_name='bcdb.QueryRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=419,
  serialized_end=433,
)


_LISTRESPONSE = _descriptor.Descriptor(
  name='ListResponse',
  full_name='bcdb.ListResponse',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
    _descriptor.FieldDescriptor(
      name='id', full_name='bcdb.ListResponse.id', index=0,
      number=1, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=435,
  serialized_end=461,
)


_FINDRESPONSE = _descriptor.Descriptor(
  name='FindResponse',
  full_name='bcdb.FindResponse',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  fields=[
    _descriptor.FieldDescriptor(
      name='id', full_name='bcdb.FindResponse.id', index=0,
      number=1, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
    _descriptor.FieldDescriptor(
      name='metadata', full_name='bcdb.FindResponse.metadata', index=1,
      number=2, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
    _descriptor.FieldDescriptor(
      name='data', full_name='bcdb.FindResponse.data', index=2,
      number=3, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=463,
  serialized_end=537,
)

_TAG.oneofs_by_name['value'].fields.append(
  _TAG.fields_by_name['string'])
_TAG.fields_by_name['string'].containing_oneof = _TAG.oneofs_by_name['value']
_TAG.oneofs_by_name['value'].fields.append(
  _TAG.fields_by_name['double'])
_TAG.fields_by_name['double'].containing_oneof = _TAG.oneofs_by_name['value']
_TAG.oneofs_by_name['value'].fields.append(
  _TAG.fields_by_name['number'])
_TAG.fields_by_name['number'].containing_oneof = _TAG.oneofs_by_name['value']
_TAG.oneofs_by_name['value'].fields.append(
  _TAG.fields_by_name['unsigned'])
_TAG.fields_by_name['unsigned'].containing_oneof = _TAG.oneofs_by_name['value']
_METADATA.fields_by_name['tags'].message_type = _TAG
_SETREQUEST.fields_by_name['metadata'].message_type = _METADATA
_GETRESPONSE.fields_by_name['metadata'].message_type = _METADATA
_UPDATEREQUEST.fields_by_name['metadata'].message_type = _METADATA
_FINDRESPONSE.fields_by_name['metadata'].message_type = _METADATA
DESCRIPTOR.message_types_by_name['Tag'] = _TAG
DESCRIPTOR.message_types_by_name['Metadata'] = _METADATA
DESCRIPTOR.message_types_by_name['SetRequest'] = _SETREQUEST
DESCRIPTOR.message_types_by_name['SetResponse'] = _SETRESPONSE
DESCRIPTOR.message_types_by_name['GetRequest'] = _GETREQUEST
DESCRIPTOR.message_types_by_name['GetResponse'] = _GETRESPONSE
DESCRIPTOR.message_types_by_name['UpdateRequest'] = _UPDATEREQUEST
DESCRIPTOR.message_types_by_name['UpdateResponse'] = _UPDATERESPONSE
DESCRIPTOR.message_types_by_name['QueryRequest'] = _QUERYREQUEST
DESCRIPTOR.message_types_by_name['ListResponse'] = _LISTRESPONSE
DESCRIPTOR.message_types_by_name['FindResponse'] = _FINDRESPONSE
_sym_db.RegisterFileDescriptor(DESCRIPTOR)

Tag = _reflection.GeneratedProtocolMessageType('Tag', (_message.Message,), {
  'DESCRIPTOR' : _TAG,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.Tag)
  })
_sym_db.RegisterMessage(Tag)

Metadata = _reflection.GeneratedProtocolMessageType('Metadata', (_message.Message,), {
  'DESCRIPTOR' : _METADATA,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.Metadata)
  })
_sym_db.RegisterMessage(Metadata)

SetRequest = _reflection.GeneratedProtocolMessageType('SetRequest', (_message.Message,), {
  'DESCRIPTOR' : _SETREQUEST,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.SetRequest)
  })
_sym_db.RegisterMessage(SetRequest)

SetResponse = _reflection.GeneratedProtocolMessageType('SetResponse', (_message.Message,), {
  'DESCRIPTOR' : _SETRESPONSE,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.SetResponse)
  })
_sym_db.RegisterMessage(SetResponse)

GetRequest = _reflection.GeneratedProtocolMessageType('GetRequest', (_message.Message,), {
  'DESCRIPTOR' : _GETREQUEST,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.GetRequest)
  })
_sym_db.RegisterMessage(GetRequest)

GetResponse = _reflection.GeneratedProtocolMessageType('GetResponse', (_message.Message,), {
  'DESCRIPTOR' : _GETRESPONSE,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.GetResponse)
  })
_sym_db.RegisterMessage(GetResponse)

UpdateRequest = _reflection.GeneratedProtocolMessageType('UpdateRequest', (_message.Message,), {
  'DESCRIPTOR' : _UPDATEREQUEST,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.UpdateRequest)
  })
_sym_db.RegisterMessage(UpdateRequest)

UpdateResponse = _reflection.GeneratedProtocolMessageType('UpdateResponse', (_message.Message,), {
  'DESCRIPTOR' : _UPDATERESPONSE,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.UpdateResponse)
  })
_sym_db.RegisterMessage(UpdateResponse)

QueryRequest = _reflection.GeneratedProtocolMessageType('QueryRequest', (_message.Message,), {
  'DESCRIPTOR' : _QUERYREQUEST,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.QueryRequest)
  })
_sym_db.RegisterMessage(QueryRequest)

ListResponse = _reflection.GeneratedProtocolMessageType('ListResponse', (_message.Message,), {
  'DESCRIPTOR' : _LISTRESPONSE,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.ListResponse)
  })
_sym_db.RegisterMessage(ListResponse)

FindResponse = _reflection.GeneratedProtocolMessageType('FindResponse', (_message.Message,), {
  'DESCRIPTOR' : _FINDRESPONSE,
  '__module__' : 'bcdb_pb2'
  # @@protoc_insertion_point(class_scope:bcdb.FindResponse)
  })
_sym_db.RegisterMessage(FindResponse)



_BCDB = _descriptor.ServiceDescriptor(
  name='BCDB',
  full_name='bcdb.BCDB',
  file=DESCRIPTOR,
  index=0,
  serialized_options=None,
  serialized_start=540,
  serialized_end=797,
  methods=[
  _descriptor.MethodDescriptor(
    name='Set',
    full_name='bcdb.BCDB.Set',
    index=0,
    containing_service=None,
    input_type=_SETREQUEST,
    output_type=_SETRESPONSE,
    serialized_options=None,
  ),
  _descriptor.MethodDescriptor(
    name='Get',
    full_name='bcdb.BCDB.Get',
    index=1,
    containing_service=None,
    input_type=_GETREQUEST,
    output_type=_GETRESPONSE,
    serialized_options=None,
  ),
  _descriptor.MethodDescriptor(
    name='Update',
    full_name='bcdb.BCDB.Update',
    index=2,
    containing_service=None,
    input_type=_UPDATEREQUEST,
    output_type=_UPDATERESPONSE,
    serialized_options=None,
  ),
  _descriptor.MethodDescriptor(
    name='List',
    full_name='bcdb.BCDB.List',
    index=3,
    containing_service=None,
    input_type=_QUERYREQUEST,
    output_type=_LISTRESPONSE,
    serialized_options=None,
  ),
  _descriptor.MethodDescriptor(
    name='Find',
    full_name='bcdb.BCDB.Find',
    index=4,
    containing_service=None,
    input_type=_QUERYREQUEST,
    output_type=_FINDRESPONSE,
    serialized_options=None,
  ),
])
_sym_db.RegisterServiceDescriptor(_BCDB)

DESCRIPTOR.services_by_name['BCDB'] = _BCDB

# @@protoc_insertion_point(module_scope)
