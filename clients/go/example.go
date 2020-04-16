package main

import (
	"context"
	"fmt"

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
	var id uint32 = 5
	fmt.Println("ID: ", id)

	result, err := cl.Get(context.TODO(), &bcdb.GetRequest{Id: id, Collection: "files"})
	if err != nil {
		panic(err)
	}

	fmt.Printf("data: %v\n", string(result.Data))
	fmt.Printf("meta: %+v\n", result.Metadata)

	// // test list
	// list, err := cl.List(context.TODO(), &bcdb.QueryRequest{})
	// if err != nil {
	// 	panic(err)
	// }

	// for {
	// 	msg, err := list.Recv()
	// 	if err == io.EOF {
	// 		break
	// 	} else if err != nil {
	// 		panic(err)
	// 	}

	// 	fmt.Println("ID: ", msg.Id)
	// }
}
