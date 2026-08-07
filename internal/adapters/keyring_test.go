//go:build darwin

package adapters

import (
	"log"
	"testing"
)

var store = KeyringSecretStore{}

func setup() func() {
	store := KeyringSecretStore{}
	store.Set("TEST_TOKEN", "TEST_VALUE")
	return func() {
		store.Delete("TEST_TOKEN")
	}
}

// test whether secret store is fetching data
func TestSecretStoreGet(t *testing.T) {
	tearDown := setup()
	value, err := store.Get("TEST_TOKEN")
	if err != nil {
		log.Println("error while fetching data from secret store")
		t.Fail()
	}
	if value != "TEST_VALUE" {
		log.Println("Incorrect value from secret store")
	}
	tearDown()
}
