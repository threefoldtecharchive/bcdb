// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait BCDB {
    fn set(&self, o: ::grpc::RequestOptions, p: super::api::Document) -> ::grpc::SingleResponse<super::api::Header>;

    fn get(&self, o: ::grpc::RequestOptions, p: super::api::Header) -> ::grpc::SingleResponse<super::api::Document>;

    fn modify(&self, o: ::grpc::RequestOptions, p: super::api::Update) -> ::grpc::SingleResponse<super::api::Header>;

    fn list(&self, o: ::grpc::RequestOptions, p: super::api::Query) -> ::grpc::StreamingResponse<super::api::Header>;

    fn find(&self, o: ::grpc::RequestOptions, p: super::api::Query) -> ::grpc::StreamingResponse<super::api::Document>;
}

// client

pub struct BCDBClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_Set: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::Document, super::api::Header>>,
    method_Get: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::Header, super::api::Document>>,
    method_Modify: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::Update, super::api::Header>>,
    method_List: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::Query, super::api::Header>>,
    method_Find: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::Query, super::api::Document>>,
}

impl ::grpc::ClientStub for BCDBClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        BCDBClient {
            grpc_client: grpc_client,
            method_Set: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/bcdb.BCDB/Set".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Get: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/bcdb.BCDB/Get".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Modify: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/bcdb.BCDB/Modify".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_List: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/bcdb.BCDB/List".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Find: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/bcdb.BCDB/Find".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl BCDB for BCDBClient {
    fn set(&self, o: ::grpc::RequestOptions, p: super::api::Document) -> ::grpc::SingleResponse<super::api::Header> {
        self.grpc_client.call_unary(o, p, self.method_Set.clone())
    }

    fn get(&self, o: ::grpc::RequestOptions, p: super::api::Header) -> ::grpc::SingleResponse<super::api::Document> {
        self.grpc_client.call_unary(o, p, self.method_Get.clone())
    }

    fn modify(&self, o: ::grpc::RequestOptions, p: super::api::Update) -> ::grpc::SingleResponse<super::api::Header> {
        self.grpc_client.call_unary(o, p, self.method_Modify.clone())
    }

    fn list(&self, o: ::grpc::RequestOptions, p: super::api::Query) -> ::grpc::StreamingResponse<super::api::Header> {
        self.grpc_client.call_server_streaming(o, p, self.method_List.clone())
    }

    fn find(&self, o: ::grpc::RequestOptions, p: super::api::Query) -> ::grpc::StreamingResponse<super::api::Document> {
        self.grpc_client.call_server_streaming(o, p, self.method_Find.clone())
    }
}

// server

pub struct BCDBServer;


impl BCDBServer {
    pub fn new_service_def<H : BCDB + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/bcdb.BCDB",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/bcdb.BCDB/Set".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.set(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/bcdb.BCDB/Get".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/bcdb.BCDB/Modify".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.modify(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/bcdb.BCDB/List".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerServerStreaming::new(move |o, p| handler_copy.list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/bcdb.BCDB/Find".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerServerStreaming::new(move |o, p| handler_copy.find(o, p))
                    },
                ),
            ],
        )
    }
}
