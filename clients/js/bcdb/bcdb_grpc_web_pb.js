/**
 * @fileoverview gRPC-Web generated client stub for bcdb
 * @enhanceable
 * @public
 */

// GENERATED CODE -- DO NOT EDIT!



const grpc = {};
grpc.web = require('grpc-web');

const proto = {};
proto.bcdb = require('./bcdb_pb.js');

/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?Object} options
 * @constructor
 * @struct
 * @final
 */
proto.bcdb.BCDBClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options['format'] = 'text';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname;

};


/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?Object} options
 * @constructor
 * @struct
 * @final
 */
proto.bcdb.BCDBPromiseClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options['format'] = 'text';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname;

};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.SetRequest,
 *   !proto.bcdb.SetResponse>}
 */
const methodDescriptor_BCDB_Set = new grpc.web.MethodDescriptor(
  '/bcdb.BCDB/Set',
  grpc.web.MethodType.UNARY,
  proto.bcdb.SetRequest,
  proto.bcdb.SetResponse,
  /**
   * @param {!proto.bcdb.SetRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.SetResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.SetRequest,
 *   !proto.bcdb.SetResponse>}
 */
const methodInfo_BCDB_Set = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.SetResponse,
  /**
   * @param {!proto.bcdb.SetRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.SetResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.SetRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.Error, ?proto.bcdb.SetResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.SetResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.bcdb.BCDBClient.prototype.set =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/bcdb.BCDB/Set',
      request,
      metadata || {},
      methodDescriptor_BCDB_Set,
      callback);
};


/**
 * @param {!proto.bcdb.SetRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.bcdb.SetResponse>}
 *     A native promise that resolves to the response
 */
proto.bcdb.BCDBPromiseClient.prototype.set =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/bcdb.BCDB/Set',
      request,
      metadata || {},
      methodDescriptor_BCDB_Set);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.GetRequest,
 *   !proto.bcdb.GetResponse>}
 */
const methodDescriptor_BCDB_Get = new grpc.web.MethodDescriptor(
  '/bcdb.BCDB/Get',
  grpc.web.MethodType.UNARY,
  proto.bcdb.GetRequest,
  proto.bcdb.GetResponse,
  /**
   * @param {!proto.bcdb.GetRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.GetResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.GetRequest,
 *   !proto.bcdb.GetResponse>}
 */
const methodInfo_BCDB_Get = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.GetResponse,
  /**
   * @param {!proto.bcdb.GetRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.GetResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.GetRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.Error, ?proto.bcdb.GetResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.GetResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.bcdb.BCDBClient.prototype.get =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/bcdb.BCDB/Get',
      request,
      metadata || {},
      methodDescriptor_BCDB_Get,
      callback);
};


/**
 * @param {!proto.bcdb.GetRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.bcdb.GetResponse>}
 *     A native promise that resolves to the response
 */
proto.bcdb.BCDBPromiseClient.prototype.get =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/bcdb.BCDB/Get',
      request,
      metadata || {},
      methodDescriptor_BCDB_Get);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.UpdateRequest,
 *   !proto.bcdb.UpdateResponse>}
 */
const methodDescriptor_BCDB_Update = new grpc.web.MethodDescriptor(
  '/bcdb.BCDB/Update',
  grpc.web.MethodType.UNARY,
  proto.bcdb.UpdateRequest,
  proto.bcdb.UpdateResponse,
  /**
   * @param {!proto.bcdb.UpdateRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.UpdateResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.UpdateRequest,
 *   !proto.bcdb.UpdateResponse>}
 */
const methodInfo_BCDB_Update = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.UpdateResponse,
  /**
   * @param {!proto.bcdb.UpdateRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.UpdateResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.UpdateRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.Error, ?proto.bcdb.UpdateResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.UpdateResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.bcdb.BCDBClient.prototype.update =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/bcdb.BCDB/Update',
      request,
      metadata || {},
      methodDescriptor_BCDB_Update,
      callback);
};


/**
 * @param {!proto.bcdb.UpdateRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.bcdb.UpdateResponse>}
 *     A native promise that resolves to the response
 */
proto.bcdb.BCDBPromiseClient.prototype.update =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/bcdb.BCDB/Update',
      request,
      metadata || {},
      methodDescriptor_BCDB_Update);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.QueryRequest,
 *   !proto.bcdb.ListResponse>}
 */
const methodDescriptor_BCDB_List = new grpc.web.MethodDescriptor(
  '/bcdb.BCDB/List',
  grpc.web.MethodType.SERVER_STREAMING,
  proto.bcdb.QueryRequest,
  proto.bcdb.ListResponse,
  /**
   * @param {!proto.bcdb.QueryRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ListResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.QueryRequest,
 *   !proto.bcdb.ListResponse>}
 */
const methodInfo_BCDB_List = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.ListResponse,
  /**
   * @param {!proto.bcdb.QueryRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ListResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.QueryRequest} request The request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.ListResponse>}
 *     The XHR Node Readable Stream
 */
proto.bcdb.BCDBClient.prototype.list =
    function(request, metadata) {
  return this.client_.serverStreaming(this.hostname_ +
      '/bcdb.BCDB/List',
      request,
      metadata || {},
      methodDescriptor_BCDB_List);
};


/**
 * @param {!proto.bcdb.QueryRequest} request The request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.ListResponse>}
 *     The XHR Node Readable Stream
 */
proto.bcdb.BCDBPromiseClient.prototype.list =
    function(request, metadata) {
  return this.client_.serverStreaming(this.hostname_ +
      '/bcdb.BCDB/List',
      request,
      metadata || {},
      methodDescriptor_BCDB_List);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.QueryRequest,
 *   !proto.bcdb.FindResponse>}
 */
const methodDescriptor_BCDB_Find = new grpc.web.MethodDescriptor(
  '/bcdb.BCDB/Find',
  grpc.web.MethodType.SERVER_STREAMING,
  proto.bcdb.QueryRequest,
  proto.bcdb.FindResponse,
  /**
   * @param {!proto.bcdb.QueryRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.FindResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.QueryRequest,
 *   !proto.bcdb.FindResponse>}
 */
const methodInfo_BCDB_Find = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.FindResponse,
  /**
   * @param {!proto.bcdb.QueryRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.FindResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.QueryRequest} request The request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.FindResponse>}
 *     The XHR Node Readable Stream
 */
proto.bcdb.BCDBClient.prototype.find =
    function(request, metadata) {
  return this.client_.serverStreaming(this.hostname_ +
      '/bcdb.BCDB/Find',
      request,
      metadata || {},
      methodDescriptor_BCDB_Find);
};


/**
 * @param {!proto.bcdb.QueryRequest} request The request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.FindResponse>}
 *     The XHR Node Readable Stream
 */
proto.bcdb.BCDBPromiseClient.prototype.find =
    function(request, metadata) {
  return this.client_.serverStreaming(this.hostname_ +
      '/bcdb.BCDB/Find',
      request,
      metadata || {},
      methodDescriptor_BCDB_Find);
};


/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?Object} options
 * @constructor
 * @struct
 * @final
 */
proto.bcdb.AclClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options['format'] = 'text';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname;

};


/**
 * @param {string} hostname
 * @param {?Object} credentials
 * @param {?Object} options
 * @constructor
 * @struct
 * @final
 */
proto.bcdb.AclPromiseClient =
    function(hostname, credentials, options) {
  if (!options) options = {};
  options['format'] = 'text';

  /**
   * @private @const {!grpc.web.GrpcWebClientBase} The client
   */
  this.client_ = new grpc.web.GrpcWebClientBase(options);

  /**
   * @private @const {string} The hostname
   */
  this.hostname_ = hostname;

};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.ACLGetRequest,
 *   !proto.bcdb.ACLGetResponse>}
 */
const methodDescriptor_Acl_Get = new grpc.web.MethodDescriptor(
  '/bcdb.Acl/Get',
  grpc.web.MethodType.UNARY,
  proto.bcdb.ACLGetRequest,
  proto.bcdb.ACLGetResponse,
  /**
   * @param {!proto.bcdb.ACLGetRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLGetResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.ACLGetRequest,
 *   !proto.bcdb.ACLGetResponse>}
 */
const methodInfo_Acl_Get = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.ACLGetResponse,
  /**
   * @param {!proto.bcdb.ACLGetRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLGetResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.ACLGetRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.Error, ?proto.bcdb.ACLGetResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.ACLGetResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.bcdb.AclClient.prototype.get =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/bcdb.Acl/Get',
      request,
      metadata || {},
      methodDescriptor_Acl_Get,
      callback);
};


/**
 * @param {!proto.bcdb.ACLGetRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.bcdb.ACLGetResponse>}
 *     A native promise that resolves to the response
 */
proto.bcdb.AclPromiseClient.prototype.get =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/bcdb.Acl/Get',
      request,
      metadata || {},
      methodDescriptor_Acl_Get);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.ACLCreateRequest,
 *   !proto.bcdb.ACLCreateResponse>}
 */
const methodDescriptor_Acl_Create = new grpc.web.MethodDescriptor(
  '/bcdb.Acl/Create',
  grpc.web.MethodType.UNARY,
  proto.bcdb.ACLCreateRequest,
  proto.bcdb.ACLCreateResponse,
  /**
   * @param {!proto.bcdb.ACLCreateRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLCreateResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.ACLCreateRequest,
 *   !proto.bcdb.ACLCreateResponse>}
 */
const methodInfo_Acl_Create = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.ACLCreateResponse,
  /**
   * @param {!proto.bcdb.ACLCreateRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLCreateResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.ACLCreateRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.Error, ?proto.bcdb.ACLCreateResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.ACLCreateResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.bcdb.AclClient.prototype.create =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/bcdb.Acl/Create',
      request,
      metadata || {},
      methodDescriptor_Acl_Create,
      callback);
};


/**
 * @param {!proto.bcdb.ACLCreateRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.bcdb.ACLCreateResponse>}
 *     A native promise that resolves to the response
 */
proto.bcdb.AclPromiseClient.prototype.create =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/bcdb.Acl/Create',
      request,
      metadata || {},
      methodDescriptor_Acl_Create);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.ACLListRequest,
 *   !proto.bcdb.ACLListResponse>}
 */
const methodDescriptor_Acl_List = new grpc.web.MethodDescriptor(
  '/bcdb.Acl/List',
  grpc.web.MethodType.SERVER_STREAMING,
  proto.bcdb.ACLListRequest,
  proto.bcdb.ACLListResponse,
  /**
   * @param {!proto.bcdb.ACLListRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLListResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.ACLListRequest,
 *   !proto.bcdb.ACLListResponse>}
 */
const methodInfo_Acl_List = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.ACLListResponse,
  /**
   * @param {!proto.bcdb.ACLListRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLListResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.ACLListRequest} request The request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.ACLListResponse>}
 *     The XHR Node Readable Stream
 */
proto.bcdb.AclClient.prototype.list =
    function(request, metadata) {
  return this.client_.serverStreaming(this.hostname_ +
      '/bcdb.Acl/List',
      request,
      metadata || {},
      methodDescriptor_Acl_List);
};


/**
 * @param {!proto.bcdb.ACLListRequest} request The request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.ACLListResponse>}
 *     The XHR Node Readable Stream
 */
proto.bcdb.AclPromiseClient.prototype.list =
    function(request, metadata) {
  return this.client_.serverStreaming(this.hostname_ +
      '/bcdb.Acl/List',
      request,
      metadata || {},
      methodDescriptor_Acl_List);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.ACLSetRequest,
 *   !proto.bcdb.ACLSetResponse>}
 */
const methodDescriptor_Acl_Set = new grpc.web.MethodDescriptor(
  '/bcdb.Acl/Set',
  grpc.web.MethodType.UNARY,
  proto.bcdb.ACLSetRequest,
  proto.bcdb.ACLSetResponse,
  /**
   * @param {!proto.bcdb.ACLSetRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLSetResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.ACLSetRequest,
 *   !proto.bcdb.ACLSetResponse>}
 */
const methodInfo_Acl_Set = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.ACLSetResponse,
  /**
   * @param {!proto.bcdb.ACLSetRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLSetResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.ACLSetRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.Error, ?proto.bcdb.ACLSetResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.ACLSetResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.bcdb.AclClient.prototype.set =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/bcdb.Acl/Set',
      request,
      metadata || {},
      methodDescriptor_Acl_Set,
      callback);
};


/**
 * @param {!proto.bcdb.ACLSetRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.bcdb.ACLSetResponse>}
 *     A native promise that resolves to the response
 */
proto.bcdb.AclPromiseClient.prototype.set =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/bcdb.Acl/Set',
      request,
      metadata || {},
      methodDescriptor_Acl_Set);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.ACLUsersRequest,
 *   !proto.bcdb.ACLUsersResponse>}
 */
const methodDescriptor_Acl_Grant = new grpc.web.MethodDescriptor(
  '/bcdb.Acl/Grant',
  grpc.web.MethodType.UNARY,
  proto.bcdb.ACLUsersRequest,
  proto.bcdb.ACLUsersResponse,
  /**
   * @param {!proto.bcdb.ACLUsersRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLUsersResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.ACLUsersRequest,
 *   !proto.bcdb.ACLUsersResponse>}
 */
const methodInfo_Acl_Grant = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.ACLUsersResponse,
  /**
   * @param {!proto.bcdb.ACLUsersRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLUsersResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.ACLUsersRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.Error, ?proto.bcdb.ACLUsersResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.ACLUsersResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.bcdb.AclClient.prototype.grant =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/bcdb.Acl/Grant',
      request,
      metadata || {},
      methodDescriptor_Acl_Grant,
      callback);
};


/**
 * @param {!proto.bcdb.ACLUsersRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.bcdb.ACLUsersResponse>}
 *     A native promise that resolves to the response
 */
proto.bcdb.AclPromiseClient.prototype.grant =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/bcdb.Acl/Grant',
      request,
      metadata || {},
      methodDescriptor_Acl_Grant);
};


/**
 * @const
 * @type {!grpc.web.MethodDescriptor<
 *   !proto.bcdb.ACLUsersRequest,
 *   !proto.bcdb.ACLUsersResponse>}
 */
const methodDescriptor_Acl_Revoke = new grpc.web.MethodDescriptor(
  '/bcdb.Acl/Revoke',
  grpc.web.MethodType.UNARY,
  proto.bcdb.ACLUsersRequest,
  proto.bcdb.ACLUsersResponse,
  /**
   * @param {!proto.bcdb.ACLUsersRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLUsersResponse.deserializeBinary
);


/**
 * @const
 * @type {!grpc.web.AbstractClientBase.MethodInfo<
 *   !proto.bcdb.ACLUsersRequest,
 *   !proto.bcdb.ACLUsersResponse>}
 */
const methodInfo_Acl_Revoke = new grpc.web.AbstractClientBase.MethodInfo(
  proto.bcdb.ACLUsersResponse,
  /**
   * @param {!proto.bcdb.ACLUsersRequest} request
   * @return {!Uint8Array}
   */
  function(request) {
    return request.serializeBinary();
  },
  proto.bcdb.ACLUsersResponse.deserializeBinary
);


/**
 * @param {!proto.bcdb.ACLUsersRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @param {function(?grpc.web.Error, ?proto.bcdb.ACLUsersResponse)}
 *     callback The callback function(error, response)
 * @return {!grpc.web.ClientReadableStream<!proto.bcdb.ACLUsersResponse>|undefined}
 *     The XHR Node Readable Stream
 */
proto.bcdb.AclClient.prototype.revoke =
    function(request, metadata, callback) {
  return this.client_.rpcCall(this.hostname_ +
      '/bcdb.Acl/Revoke',
      request,
      metadata || {},
      methodDescriptor_Acl_Revoke,
      callback);
};


/**
 * @param {!proto.bcdb.ACLUsersRequest} request The
 *     request proto
 * @param {?Object<string, string>} metadata User defined
 *     call metadata
 * @return {!Promise<!proto.bcdb.ACLUsersResponse>}
 *     A native promise that resolves to the response
 */
proto.bcdb.AclPromiseClient.prototype.revoke =
    function(request, metadata) {
  return this.client_.unaryCall(this.hostname_ +
      '/bcdb.Acl/Revoke',
      request,
      metadata || {},
      methodDescriptor_Acl_Revoke);
};


module.exports = proto.bcdb;

