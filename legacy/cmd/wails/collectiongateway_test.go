package main

import (
	"testing"
	"xrest/internal/models"
)

func TestCollectionGateway(t *testing.T) {
	cg := NewCollectionGateway()

	cols, err := cg.LoadCollections()
	if err != nil {
		t.Fatalf("expected no error loading collections, got %v", err)
	}
	if len(cols) != 0 {
		t.Errorf("expected 0 loaded collections, got %d", len(cols))
	}

	testCols := []models.Service{
		{ID: "c1", Name: "Test Collection"},
	}
	saved, err := cg.SaveCollections(testCols)
	if err != nil {
		t.Fatalf("expected no error saving collections, got %v", err)
	}
	if len(saved) != 1 || saved[0].ID != "c1" {
		t.Errorf("expected saved collections to match input, got %v", saved)
	}
}
