package main

import (
	"log"
	"xrest/internal/models"
)

// CollectionGateway handles collection data storage and retrieval.
type CollectionGateway struct{}

// NewCollectionGateway creates a new CollectionGateway.
func NewCollectionGateway() *CollectionGateway {
	return &CollectionGateway{}
}

// LoadCollections returns all stored collections.
func (c *CollectionGateway) LoadCollections() ([]models.Service, error) {
	log.Println("[CollectionGateway] LoadCollections called")
	return []models.Service{}, nil
}

// SaveCollections persists collections.
func (c *CollectionGateway) SaveCollections(collections []models.Service) ([]models.Service, error) {
	log.Printf("[CollectionGateway] SaveCollections called with %d collections\n", len(collections))
	return collections, nil
}
