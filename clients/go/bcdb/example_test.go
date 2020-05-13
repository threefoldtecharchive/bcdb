package bcdb

import (
	"context"
	"fmt"
	"io"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	proto "github.com/threefoldtech/dbreboot/clients/go/bcdb/bcdb_proto"
	"google.golang.org/grpc"
)

func ExampleClient(t *testing.T) {
	client, err := New(1, "lift round pool release tape mechanic enter chase immune collect light swear like silly obtain resemble vanish degree sail hair drift eyebrow property double", grpc.WithInsecure())
	require.NoError(t, err)

	ctx := context.Background()
	col := client.Collection("test")

	data := []byte("hello world")
	id, err := col.Create(ctx, data, []Tag{
		{
			Key:   "foo",
			Value: "bar",
		},
	})
	require.NoError(t, err)

	t.Logf("id created %d\n", id)

	iter, err := col.Iter(ctx)
	require.NoError(t, err)

	for iter.Next() {
		item, err := iter.Item()
		if err != nil {
			t.FailNow()
		}

		t.Logf("%d %s %+v\n", item.ID(), string(item.Value()), item.Tags())
	}
}

func ExampleRawGRPC() {

	const (
		userID   = 6
		mnemonic = "hawk flush rifle build globe festival process enrich angry okay inmate pilot reunion february best health pigeon actor spare absurd glory ahead situate float"
	)

	auth, err := NewBCDBAuthenticatorFromMnemonic(userID, mnemonic, WithExpiresDuration(3*time.Second))
	if err != nil {
		panic(err)
	}

	client, err := grpc.Dial("localhost:50051", grpc.WithInsecure(), grpc.WithPerRPCCredentials(auth))
	if err != nil {
		panic(err)
	}

	cl := proto.NewBCDBClient(client)

	name := fmt.Sprintf("test-file-name-%d", time.Now().Unix())
	req := proto.SetRequest{
		Data: []byte("hello world"),
		Metadata: &proto.Metadata{
			Acl:        &proto.AclRef{proto.Acl: 100},
			Collection: "files",
			Tags: []*proto.Tag{
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

	list, err := cl.Find(context.TODO(), &proto.QueryRequest{
		Collection: "files",
		Tags: []*proto.Tag{
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
