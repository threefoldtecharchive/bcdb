package main

import (
	"context"
	"fmt"
	"io"

	"time"

	"example.com/test/bcdb"
	"google.golang.org/grpc"
)

type Authenticator struct {
}

func (a *Authenticator) GetRequestMetadata(ctx context.Context, uri ...string) (map[string]string, error) {
	s := `Signature keyId="6",algorithm="hs2019",created="%d",expires="%d",headers="(created) (expires) (key-id)", signature="Base64(RSA-SHA512(signing string))"`
	n := time.Now()
	s = fmt.Sprintf(s, n.Unix(), n.Add(3*time.Second).Unix())

	values := map[string]string{
		"Authorization": s,
	}
	return values, nil
}

func (a *Authenticator) RequireTransportSecurity() bool {
	return false
}

func main() {
	client, err := grpc.Dial("localhost:50051", grpc.WithInsecure(), grpc.WithPerRPCCredentials(&Authenticator{}))
	if err != nil {
		panic(err)
	}

	cl := bcdb.NewBCDBClient(client)

	// name := fmt.Sprintf("test-file-name-%d", time.Now().Unix())
	// req := bcdb.SetRequest{
	// 	Data: []byte("hello world"),
	// 	Metadata: &bcdb.Metadata{
	// 		Acl:        &bcdb.AclRef{Acl: 100},
	// 		Collection: "files",
	// 		Tags: []*bcdb.Tag{
	// 			{
	// 				Key:   "name",
	// 				Value: name,
	// 			},
	// 			{
	// 				Key:   "dir",
	// 				Value: "/path/to/file",
	// 			},
	// 			{
	// 				Key:   "type",
	// 				Value: "file",
	// 			},
	// 		},
	// 	},
	// }

	// response, err := cl.Set(context.TODO(), &req)
	// if err != nil {
	// 	panic(err)
	// }

	// id := response.GetId()
	// fmt.Println("ID:", id)

	list, err := cl.Find(context.TODO(), &bcdb.QueryRequest{
		Collection: "files",
		Tags: []*bcdb.Tag{
			{Key: "type", Value: "file"},
		},
	})
	if err != nil {
		panic(err)
	}

	for {
		obj, err := list.Recv()
		if err == io.EOF {
			break
		} else if err != nil {
			panic(err)
		}

		fmt.Printf("ID: %+v\n", obj)
	}

}
