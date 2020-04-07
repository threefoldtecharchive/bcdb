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


module.exports = proto.bcdb;

