# Example grpc web setup for bcdb
## install deps
```bash
npm install
```
## build example.js
this file has example code to how to use the client
```bash
npx webpack example.js
```

## run envoy
Run envoy proxy
```bash
docker run --rm -t -v $(pwd)/envoy.yaml:/etc/envoy/envoy.yaml --network host --name envoy envoyproxy/envoy:v1.13.1
```
Note we use host networking for demonstration only, since we will run bcdb grpc service on localhost this makes configuration for envoy easier (check envoy.yaml)

## make sure bcdb is running
start bcdb service

## finally host the js files
```bash
cd js/bcdb
python -m http.server
```

## open localhost:8000 in browser
once there, open developer console and check the logs
### what is going on
- We built an example code (example.js) that uses the generated client (bcdb_bp.js & bcdb_grp_web_bp.js)
- The code connects to localhost:8080 (envoy endpoint)
- The envoy is configured to contact bcdb (check envoy.yaml)
- The example code makes a simple call, and gets the results from bcdb
