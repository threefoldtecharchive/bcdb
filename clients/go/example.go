package main

import (
	"context"
	"fmt"
	"io"

	"example.com/test/bcdb"
	"google.golang.org/grpc"
)

func main() {
	client, err := grpc.Dial("localhost:50051", grpc.WithInsecure())
	if err != nil {
		panic(err)
	}

	cl := bcdb.NewBCDBClient(client)

	req := bcdb.SetRequest{
		Data: []byte("hello world"),
		Metadata: &bcdb.Metadata{
			Tags: []*bcdb.Tag{
				&bcdb.Tag{
					Key:   "name",
					Value: &bcdb.Tag_String_{String_: "test"},
				},
			},
		},
	}

	response, err := cl.Set(context.TODO(), &req)
	if err != nil {
		panic(err)
	}

	fmt.Println(response.GetId())

	// test list
	list, err := cl.List(context.TODO(), &bcdb.QueryRequest{})
	if err != nil {
		panic(err)
	}

	for {
		msg, err := list.Recv()
		if err == io.EOF {
			break
		} else if err != nil {
			panic(err)
		}

		fmt.Println("ID: ", msg.Id)
	}
}
