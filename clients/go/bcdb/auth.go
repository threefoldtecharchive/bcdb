package bcdb

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"encoding/base64"
	"fmt"
	"time"

	"github.com/tyler-smith/go-bip39"
	"google.golang.org/grpc/credentials"
)

// AuthOption set some optional config on the authenticator
type AuthOption interface {
	set(*authImpl)
}

type authOptionFn func(*authImpl)

func (f authOptionFn) set(a *authImpl) {
	f(a)
}

// WithExpiresDuration sets expiration duration
func WithExpiresDuration(d time.Duration) AuthOption {
	return authOptionFn(func(a *authImpl) {
		a.expires = d
	})
}

type authImpl struct {
	id      uint64
	privKey ed25519.PrivateKey
	expires time.Duration
}

// NewBCDBAuthenticator creates a new credentials.PerRPCCredentials
func NewBCDBAuthenticator(id uint64, seed []byte, opt ...AuthOption) (credentials.PerRPCCredentials, error) {
	if len(seed) != ed25519.SeedSize {
		return nil, fmt.Errorf("seed has the wrong size %d", len(seed))
	}

	privateKey := ed25519.NewKeyFromSeed(seed)

	a := &authImpl{
		id:      id,
		privKey: privateKey,
		expires: 3 * time.Second,
	}

	for _, o := range opt {
		o.set(a)
	}

	return a, nil
}

// NewBCDBAuthenticatorFromMnemonic creates a new credentials.PerRPCCredentials
func NewBCDBAuthenticatorFromMnemonic(id uint64, mnemonic string, opt ...AuthOption) (credentials.PerRPCCredentials, error) {
	seed, err := bip39.EntropyFromMnemonic(mnemonic)
	if err != nil {
		return nil, err
	}

	return NewBCDBAuthenticator(id, seed, opt...)

}

func (a *authImpl) GetRequestMetadata(ctx context.Context, uri ...string) (map[string]string, error) {
	created := time.Now()
	expires := created.Add(a.expires)
	var buf bytes.Buffer

	buf.WriteString(fmt.Sprintf("(created): %d\n", created.Unix()))
	buf.WriteString(fmt.Sprintf("(expires): %d\n", expires.Unix()))
	buf.WriteString(fmt.Sprintf("(key-id): %d", a.id))

	signature := base64.RawStdEncoding.EncodeToString(ed25519.Sign(a.privKey, buf.Bytes()))

	s := `Signature keyId="%d",algorithm="hs2019",created="%d",expires="%d",headers="(created) (expires) (key-id)", signature="%s"`

	s = fmt.Sprintf(s, a.id, created.Unix(), expires.Unix(), signature)

	values := map[string]string{
		"Authorization": s,
	}

	return values, nil
}

func (a *authImpl) RequireTransportSecurity() bool {
	return false
}
