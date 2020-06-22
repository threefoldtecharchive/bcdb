package main

import (
	"context"
	"fmt"
	"io"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/pkg/errors"
	"github.com/threefoldtech/bcdb/clients/go/bcdb"
)

var (
	ErrNotFound = fmt.Errorf("not found")
)

type FileType string

const (
	File      FileType = "file"
	Directory FileType = "dir"

	tagName = "name"
	tagDir  = "dir"
	tagType = "type"
)

type Cursor struct {
	cl bcdb.BCDB_FindClient
}

func (c *Cursor) Recv() (Metadata, error) {
	r, err := c.cl.Recv()
	if err != nil {
		return Metadata{}, err
	}

	return NewMetadata(r.Id, r.Metadata.Tags), nil
}

func put(ctx context.Context, cl bcdb.BCDBClient, path string, data []byte) error {
	abs := filepath.Join("/", filepath.Clean(path))
	dir := filepath.Dir(abs)
	name := filepath.Base(abs)

	if err := mkdirAll(ctx, cl, dir); err != nil {
		return err
	}

	info, err := get(ctx, cl, dir, name)
	if err == nil {
		//file exists
		if info.Type() != File {
			return fmt.Errorf("'%s' is not a file", path)
		}

		_, err := cl.Update(ctx, &bcdb.UpdateRequest{
			Id: info.id,
			Metadata: &bcdb.Metadata{
				Collection: Collection,
				Tags: []*bcdb.Tag{
					{Key: tagType, Value: string(File)},
					{Key: tagName, Value: name},
					{Key: tagDir, Value: dir},
				},
			},
			Data: &bcdb.UpdateRequest_UpdateData{
				Data: data,
			},
		})

		return err

	} else if err == ErrNotFound {
		// set content here

		_, err := cl.Set(ctx, &bcdb.SetRequest{
			Metadata: &bcdb.Metadata{
				Collection: Collection,
				Tags: []*bcdb.Tag{
					{Key: tagType, Value: string(File)},
					{Key: tagName, Value: name},
					{Key: tagDir, Value: dir},
				},
			},
			Data: data,
		})

		return err
	} else {
		return err
	}

	return err
}

func mkdirAll(ctx context.Context, cl bcdb.BCDBClient, dir string) error {
	dir = filepath.Join("/", filepath.Clean(dir))
	parts := strings.Split(dir, "/")
	current := "/"
	for _, part := range parts {
		if len(part) == 0 {
			//do nothing, skip
			continue
		}

		info, err := get(context.Background(), cl, current, part)
		if err == ErrNotFound {
			if err := mkdir(context.Background(), cl, current, part); err != nil {
				return errors.Wrap(err, "failed to create directory")
			}
		} else if err != nil {
			return errors.Wrap(err, "failed list directory")
		} else {
			if info.Type() != Directory {
				return errors.Wrapf(err, "path '%s' is not a directory", filepath.Join(current, part))
			}
		}

		current = filepath.Join(current, part)
	}
	return nil
}

func mkdir(ctx context.Context, cl bcdb.BCDBClient, dir, name string) error {
	_, err := cl.Set(ctx, &bcdb.SetRequest{
		Metadata: &bcdb.Metadata{
			Collection: Collection,
			Tags: []*bcdb.Tag{
				{Key: tagType, Value: string(Directory)},
				{Key: tagName, Value: name},
				{Key: tagDir, Value: dir},
			},
		},
	})

	return err
}

func download(ctx context.Context, cl bcdb.BCDBClient, dir, name string) ([]byte, error) {
	info, err := get(ctx, cl, dir, name)
	if err != nil {
		return nil, err
	}

	if info.Type() != File {
		return nil, fmt.Errorf("invalid file type")
	}

	response, err := cl.Get(ctx, &bcdb.GetRequest{
		Collection: Collection,
		Id:         info.ID(),
	})

	if err != nil {
		return nil, errors.Wrap(err, "failed to get file")
	}

	return response.Data, nil
}

func get(ctx context.Context, cl bcdb.BCDBClient, dir, name string) (Metadata, error) {
	dir = filepath.Clean(filepath.Join("/", dir))
	if name == "" {
		return Metadata{}, fmt.Errorf("missing filename")
	}
	results, err := cl.Find(ctx, &bcdb.QueryRequest{
		Collection: Collection,
		Tags: []*bcdb.Tag{
			{Key: tagDir, Value: dir},
			{Key: tagName, Value: name},
		},
	})

	if err != nil {
		return Metadata{}, errors.Wrap(err, "failed to list files")
	}

	cur := Cursor{cl: results}
	meta, err := cur.Recv()
	if err == io.EOF {
		return Metadata{}, ErrNotFound
	}

	return meta, err
}

func list(ctx context.Context, cl bcdb.BCDBClient, dir string) (*Cursor, error) {
	dir = filepath.Clean(filepath.Join("/", dir))

	results, err := cl.Find(ctx, &bcdb.QueryRequest{
		Collection: Collection,
		Tags: []*bcdb.Tag{
			{Key: tagDir, Value: dir},
		},
	})

	if err != nil {
		return nil, errors.Wrap(err, "failed to list files")
	}

	return &Cursor{cl: results}, nil

}

type Metadata struct {
	id   uint32
	tags map[string]string
}

func NewMetadata(id uint32, tags []*bcdb.Tag) Metadata {
	m := Metadata{
		id:   id,
		tags: make(map[string]string),
	}
	for _, tag := range tags {
		m.tags[tag.Key] = tag.Value
	}

	return m
}

func (m Metadata) ID() uint32 {
	return m.id
}

func (m Metadata) Created() time.Time {
	v, ok := m.tags[":created"]
	if !ok {
		return time.Unix(0, 0)
	}

	created, _ := strconv.ParseInt(v, 10, 64)
	return time.Unix(created, 0)
}

func (m Metadata) Base() string {
	return m.tags[tagName]
}

func (m Metadata) Dir() string {
	return m.tags[tagDir]
}

func (m Metadata) Type() FileType {
	return FileType(m.tags[tagType])
}

func (m Metadata) Size() uint64 {
	v, ok := m.tags[":size"]
	if !ok {
		return 0
	}

	size, _ := strconv.ParseUint(v, 10, 64)
	return size
}
