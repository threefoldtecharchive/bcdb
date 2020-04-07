
console.log("testing...");

const { GetRequest } = require('./bcdb_pb.js');
const { BCDBClient } = require('./bcdb_grpc_web_pb.js');

var client = new BCDBClient('http://localhost:8080');

var request = new GetRequest();
request.setId('some id');

client.get(request, {}, function (err, response) {
    // ...
    console.log(err, response);
});
