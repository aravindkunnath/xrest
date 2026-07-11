package adapters

import (
	"os"
	"sync"

	"github.com/zalando/go-keyring"
)

const serviceName = "xrest"

// SecretStore defines the interface for secure storage
type SecretStore interface {
	Get(key string) (string, error)
	Set(key, value string) error
	Delete(key string) error
}

// KeyringSecretStore implements SecretStore using OS Keyring
type KeyringSecretStore struct{}

func (s *KeyringSecretStore) Get(key string) (string, error) {
	return keyring.Get(serviceName, key)
}

func (s *KeyringSecretStore) Set(key, value string) error {
	return keyring.Set(serviceName, key, value)
}

func (s *KeyringSecretStore) Delete(key string) error {
	return keyring.Delete(serviceName, key)
}

// InMemorySecretStore provides an in-memory fallback for testing/CI environments
type InMemorySecretStore struct {
	mu   sync.RWMutex
	data map[string]string
}

func NewInMemorySecretStore() *InMemorySecretStore {
	return &InMemorySecretStore{
		data: make(map[string]string),
	}
}

func (s *InMemorySecretStore) Get(key string) (string, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	val, ok := s.data[key]
	if !ok {
		return "", keyring.ErrNotFound
	}
	return val, nil
}

func (s *InMemorySecretStore) Set(key, value string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.data[key] = value
	return nil
}

func (s *InMemorySecretStore) Delete(key string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, ok := s.data[key]; !ok {
		return keyring.ErrNotFound
	}
	delete(s.data, key)
	return nil
}

// GetSecretStore returns the appropriate store depending on the environment
func GetSecretStore() SecretStore {
	if os.Getenv("XREST_ENV") == "test" || os.Getenv("XREST_ENV") == "ci" {
		return NewInMemorySecretStore()
	}
	return &KeyringSecretStore{}
}
