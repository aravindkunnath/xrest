package adapters

import (
	"errors"
	"sync"
	"time"
)

type TokenCache struct {
	tokenCache sync.Map // map[string]cachedToken
}

type cachedToken struct {
	token     string
	expiresAt time.Time
}

func (cache *TokenCache) Get(key string) (cachedToken, error) {
	res, ok := cache.tokenCache.Load(key)
	if !ok {
		return cachedToken{}, errors.New("Key not found in token")
	}
	return res.(cachedToken), nil
}

func (cache *TokenCache) Put(key string, value cachedToken) {
	cache.tokenCache.Store(key, value)
}
