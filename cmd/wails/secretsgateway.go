package main

import (
	"encoding/json"
	"io"
	"log"
	"os"
	"path/filepath"
	"sync"
	"xrest/internal/adapters"

	"github.com/adrg/xdg"
)

// SecretsGateway handles secure storing, listing, and removing of workspace secrets.
type SecretsGateway struct {
	mu         sync.Mutex
	store      adapters.SecretStore
	configPath string
}

// NewSecretsGateway creates a new SecretsGateway instance.
func NewSecretsGateway() *SecretsGateway {
	// Locate/create paths using XDG Base Directory Specification
	var configPath string
	if os.Getenv("XREST_ENV") == "test" {
		configPath = filepath.Join(os.TempDir(), "xrest-test", "secrets.json")
	} else {
		configPath = filepath.Join(xdg.ConfigHome, "xrest", "secrets.json")
	}

	return &SecretsGateway{
		store:      adapters.GetSecretStore(),
		configPath: configPath,
	}
}

// loadKeys reads the secrets.json file to list the saved keys.
func (g *SecretsGateway) loadKeys() ([]string, error) {
	if _, err := os.Stat(g.configPath); os.IsNotExist(err) {
		return []string{}, nil
	}

	file, err := os.Open(g.configPath)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	bytes, err := io.ReadAll(file)
	if err != nil {
		return nil, err
	}

	var keys []string
	if err := json.Unmarshal(bytes, &keys); err != nil {
		return []string{}, nil // Return empty list on parse error or empty file
	}

	return keys, nil
}

// saveKeys writes the list of secret keys back to secrets.json.
func (g *SecretsGateway) saveKeys(keys []string) error {
	dir := filepath.Dir(g.configPath)
	if err := os.MkdirAll(dir, 0700); err != nil {
		return err
	}

	bytes, err := json.Marshal(keys)
	if err != nil {
		return err
	}

	return os.WriteFile(g.configPath, bytes, 0600)
}

// GetSecrets returns a list of all defined secret keys.
func (g *SecretsGateway) GetSecrets() ([]string, error) {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Println("[SecretsGateway] GetSecrets called")
	return g.loadKeys()
}

// AddSecret saves the secret value securely and appends the key name to the list of secret keys.
func (g *SecretsGateway) AddSecret(key, value string) ([]string, error) {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Printf("[SecretsGateway] AddSecret called for key: %s\n", key)

	// Save to secure keyring
	if err := g.store.Set(key, value); err != nil {
		return nil, err
	}

	// Update configuration keys file
	keys, err := g.loadKeys()
	if err != nil {
		return nil, err
	}

	exists := false
	for _, k := range keys {
		if k == key {
			exists = true
			break
		}
	}

	if !exists {
		keys = append(keys, key)
		if err := g.saveKeys(keys); err != nil {
			return nil, err
		}
	}

	return keys, nil
}

// DeleteSecret removes the secret from the keyring and removes its key name from the configuration list.
func (g *SecretsGateway) DeleteSecret(key string) ([]string, error) {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Printf("[SecretsGateway] DeleteSecret called for key: %s\n", key)

	// Delete from secure keyring
	_ = g.store.Delete(key) // Ignore error if not found in keyring to proceed with key list cleanup

	// Update configuration keys file
	keys, err := g.loadKeys()
	if err != nil {
		return nil, err
	}

	newKeys := []string{}
	for _, k := range keys {
		if k != key {
			newKeys = append(newKeys, k)
		}
	}

	if err := g.saveKeys(newKeys); err != nil {
		return nil, err
	}

	return newKeys, nil
}

// GetSecret retrieves the decrypted secret value from the keyring.
func (g *SecretsGateway) GetSecret(key string) (string, error) {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Printf("[SecretsGateway] GetSecret called for key: %s\n", key)
	return g.store.Get(key)
}
