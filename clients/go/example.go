package main

import (
	"context"
	"fmt"
	"io"

	"time"

	"github.com/threefoldtech/dbreboot/clients/go/bcdb"
	"google.golang.org/grpc"
)

func main() {
	const (
		userID   = 6
		mnemonic = "hawk flush rifle build globe festival process enrich angry okay inmate pilot reunion february best health pigeon actor spare absurd glory ahead situate float"
	)

	auth, err := bcdb.NewBCDBAuthenticatorFromMnemonic(userID, mnemonic, bcdb.WithExpiresDuration(3*time.Second))
	if err != nil {
		panic(err)
	}

	client, err := grpc.Dial("localhost:50051", grpc.WithInsecure(), grpc.WithPerRPCCredentials(auth))
	if err != nil {
		panic(err)
	}

	cl := bcdb.NewBCDBClient(client)

	name := fmt.Sprintf("test-file-name-%d", time.Now().Unix())
	req := bcdb.SetRequest{
		Data: []byte("hello world"),
		Metadata: &bcdb.Metadata{
			Acl:        &bcdb.AclRef{Acl: 100},
			Collection: "files",
			Tags: []*bcdb.Tag{
				{
					Key:   "name",
					Value: name,
				},
				{
					Key:   "dir",
					Value: "/path/to/file",
				},
				{
					Key:   "type",
					Value: "file",
				},
			},
		},
	}

	response, err := cl.Set(context.TODO(), &req)
	if err != nil {
		panic(err)
	}

	id := response.GetId()
	fmt.Println("ID:", id)

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
