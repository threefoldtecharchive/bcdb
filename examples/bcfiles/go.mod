module github.com/threefoldtech/dbreboot/examples/bcfiles

go 1.14

require (
	github.com/coreos/go-semver v0.3.0
	github.com/golang/protobuf v1.4.0 // indirect
	github.com/minio/cli v1.22.0
	github.com/pkg/errors v0.9.1
	github.com/rs/zerolog v1.18.0
	github.com/threefoldtech/dbreboot/clients/go v0.0.0-20200429124008-9ae32c83752b
	github.com/urfave/cli v1.22.4 // indirect
	golang.org/x/crypto v0.0.0-20200427165652-729f1e841bcc // indirect
	golang.org/x/net v0.0.0-20200425230154-ff2c4b7c35a0 // indirect
	golang.org/x/sys v0.0.0-20200428200454-593003d681fa // indirect
	golang.org/x/text v0.3.2 // indirect
	google.golang.org/genproto v0.0.0-20200429120912-1f37eeb960b2 // indirect
	google.golang.org/grpc v1.29.1
)

replace github.com/threefoldtech/dbreboot/clients/go => ../../clients/go
