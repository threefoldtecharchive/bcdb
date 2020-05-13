package bcdb

import (
	"context"
	"io"
	"time"

	proto "github.com/threefoldtech/dbreboot/clients/go/bcdb/bcdb_proto"

	"google.golang.org/grpc"
)

type Client struct {
	grpc proto.BCDBClient
}

// New creates a new BCDB client
func New(userID uint64, mnemonic string, opts ...grpc.DialOption) (*Client, error) {
	auth, err := NewBCDBAuthenticatorFromMnemonic(userID, mnemonic, WithExpiresDuration(3*time.Second))
	if err != nil {
		return nil, err
	}

	opts = append(opts, grpc.WithPerRPCCredentials(auth))
	client, err := grpc.Dial("localhost:50051", opts...)
	if err != nil {
		return nil, err
	}

	return &Client{
		grpc: proto.NewBCDBClient(client),
	}, nil
}

type Collection struct {
	client proto.BCDBClient
	name   string
}

type Tag struct {
	Key   string
	Value string
}

func (c *Client) Collection(name string) *Collection {
	return &Collection{
		client: c.grpc,
		name:   name,
	}
}

func (c *Collection) Create(ctx context.Context, data []byte, tags []Tag) (uint32, error) { //TODO add ACL
	req := proto.SetRequest{
		Data: data,
		Metadata: &proto.Metadata{
			// Acl:        &AclRef{Acl: 100},
			Collection: c.name,
			Tags: func(tags []Tag) []*proto.Tag {
				r := make([]*proto.Tag, len(tags))
				for i := range tags {
					r[i] = &proto.Tag{
						Key:   tags[i].Key,
						Value: tags[i].Value,
					}
				}
				return r
			}(tags),
		},
	}

	resp, err := c.client.Set(ctx, &req)
	if err != nil {
		return 0, err
	}
	return resp.GetId(), nil
}

func (c *Collection) Get(ctx context.Context, id uint32) ([]byte, error) {
	resp, err := c.client.Get(ctx, &proto.GetRequest{
		Id:         id,
		Collection: c.name,
	})
	if err != nil {
		return nil, err
	}

	return resp.GetData(), nil
}

type Iterator struct {
	collection *Collection
	list       proto.BCDB_ListClient

	currentID uint32
}

func (i *Iterator) Item() (*Item, error) {
	resp, err := i.collection.client.Get(context.TODO(), &proto.GetRequest{
		Id:         i.currentID,
		Collection: i.collection.name,
	})
	if err != nil {
		return nil, err
	}

	return &Item{
		id:   i.currentID,
		item: resp,
	}, nil
}

func (i *Iterator) Next() bool {
	msg, err := i.list.Recv()
	if err == io.EOF {
		return false
	}
	i.currentID = msg.GetId()
	return true
}

type Item struct {
	id   uint32
	item *proto.GetResponse
}

func (i *Item) ID() uint32 {
	return i.id
}

func (i *Item) Value() []byte {
	return i.item.GetData()
}

func (i *Item) Tags() []Tag {
	tags := make([]Tag, len(i.item.GetMetadata().Tags))
	for y := range i.item.GetMetadata().Tags {
		tags[y] = Tag{
			Key:   i.item.GetMetadata().Tags[y].GetKey(),
			Value: i.item.GetMetadata().Tags[y].GetValue(),
		}
	}
	return tags
}

func (c *Collection) Iter(ctx context.Context) (*Iterator, error) {
	list, err := c.client.List(ctx, &proto.QueryRequest{Collection: c.name})
	if err != nil {
		return nil, err
	}

	return &Iterator{
		collection: c,
		list:       list,
	}, nil
}
