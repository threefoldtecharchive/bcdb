package main

import (
	"context"
	"crypto/ed25519"
	"encoding/hex"
	"fmt"
	"io"
	"os"

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

	identity := bcdb.NewIdentityClient(client)
	signed, err := identity.Sign(context.Background(), &bcdb.SignRequest{
		Message: []byte("hello world"),
	})

	if err != nil {
		panic(err)
	}

	fmt.Println(signed.Identity.GetId(), signed.Identity.GetKey())
	key, err := hex.DecodeString(signed.Identity.GetKey())
	if err != nil {
		panic(err)
	}
	pk := ed25519.PublicKey(key)
	fmt.Println("valid:", ed25519.Verify(pk, []byte("hello world"), signed.Signature))

	//signed.Signature
	os.Exit(0)

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
