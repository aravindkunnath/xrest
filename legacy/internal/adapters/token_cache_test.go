package adapters

import (
	"sync"
	"testing"
	"time"
)

func TestTokenCache_Get_MissingKey(t *testing.T) {
	c := TokenCache{}
	_, err := c.Get("non-existent")
	if err == nil {
		t.Error("Expected error when getting non-existent key, but got nil")
	}
	expectedErrMsg := "Key not found in token"
	if err.Error() != expectedErrMsg {
		t.Errorf("Expected error message %q, got %q", expectedErrMsg, err.Error())
	}
}

func TestTokenCache_PutAndGet(t *testing.T) {
	c := TokenCache{}
	now := time.Now()
	token := cachedToken{
		token:     "my-token-123",
		expiresAt: now,
	}

	c.Put("user1", token)

	retrieved, err := c.Get("user1")
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}

	if retrieved.token != token.token {
		t.Errorf("Expected token %q, got %q", token.token, retrieved.token)
	}

	if !retrieved.expiresAt.Equal(token.expiresAt) {
		t.Errorf("Expected expiresAt %v, got %v", token.expiresAt, retrieved.expiresAt)
	}
}

func TestTokenCache_Overwrite(t *testing.T) {
	c := TokenCache{}
	token1 := cachedToken{
		token:     "token-1",
		expiresAt: time.Now(),
	}
	token2 := cachedToken{
		token:     "token-2",
		expiresAt: time.Now().Add(time.Hour),
	}

	c.Put("key", token1)
	c.Put("key", token2)

	retrieved, err := c.Get("key")
	if err != nil {
		t.Fatalf("Unexpected error: %v", err)
	}

	if retrieved.token != token2.token {
		t.Errorf("Expected token %q after overwrite, got %q", token2.token, retrieved.token)
	}
}

func TestTokenCache_Concurrent(t *testing.T) {
	c := TokenCache{}
	const workers = 20
	const operationsPerWorker = 100

	var wg sync.WaitGroup
	wg.Add(workers * 2)

	// Concurrent Writers
	for i := 0; i < workers; i++ {
		go func(workerID int) {
			defer wg.Done()
			for j := 0; j < operationsPerWorker; j++ {
				c.Put("key", cachedToken{
					token:     "token",
					expiresAt: time.Now(),
				})
			}
		}(i)
	}

	// Concurrent Readers
	for i := 0; i < workers; i++ {
		go func(workerID int) {
			defer wg.Done()
			for j := 0; j < operationsPerWorker; j++ {
				_, _ = c.Get("key")
			}
		}(i)
	}

	wg.Wait()
}
