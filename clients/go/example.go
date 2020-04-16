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

	// req := bcdb.SetRequest{
	// 	Data: []byte("hello world"),
	// 	Metadata: &bcdb.Metadata{
	// 		Collection: "files",
	// 		Tags: []*bcdb.Tag{
	// 			&bcdb.Tag{
	// 				Key:   "name",
	// 				Value: "azmy",
	// 			},
	// 			&bcdb.Tag{
	// 				Key:   "dir",
	// 				Value: "/path/to/file",
	// 			},
	// 			&bcdb.Tag{
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

	list, err := cl.List(context.TODO(), &bcdb.QueryRequest{
		Collection: "files",
	})
	if err != nil {
		panic(err)
	}

	for {
		fmt.Println("doing receive")
		obj, err := list.Recv()
		if err == io.EOF {
			break
		} else if err != nil {
			panic(err)
		}

		fmt.Println("ID: ", obj.Id)
	}

}
