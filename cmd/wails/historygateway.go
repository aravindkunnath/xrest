package main

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"sync"
	"time"
	"xrest/internal/adapters"
	"xrest/internal/models"
)

type HistoryGateway struct {
	mu     sync.Mutex
	repo   *adapters.SqliteHistoryRepository
	dbPath string
}

func NewHistoryGateway() *HistoryGateway {
	var dbPath string
	if os.Getenv("XREST_ENV") == "test" {
		dbPath = filepath.Join(os.TempDir(), "xrest-test", "history.db")
	} else {
		dbPath = filepath.Join(os.Getenv("HOME"), ".xrest", "history.db")
	}

	return &HistoryGateway{
		dbPath: dbPath,
	}
}

func (g *HistoryGateway) getRepo() (*adapters.SqliteHistoryRepository, error) {
	if g.repo != nil {
		return g.repo, nil
	}

	repo, err := adapters.NewSqliteHistoryRepository(g.dbPath)
	if err != nil {
		return nil, err
	}
	g.repo = repo
	return g.repo, nil
}

func (g *HistoryGateway) GetHistory(limit, offset int) ([]models.HistoryEntry, error) {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Printf("[HistoryGateway] GetHistory called with limit %d, offset %d\n", limit, offset)
	repo, err := g.getRepo()
	if err != nil {
		return nil, fmt.Errorf("failed to get repository: %w", err)
	}

	return repo.GetHistory(limit, offset)
}

func (g *HistoryGateway) AddHistory(entry models.HistoryEntry) (models.HistoryEntry, error) {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Printf("[HistoryGateway] AddHistory called for URL: %s\n", entry.URL)
	repo, err := g.getRepo()
	if err != nil {
		return entry, fmt.Errorf("failed to get repository: %w", err)
	}

	if entry.ID == "" {
		b := make([]byte, 8)
		_, _ = rand.Read(b)
		entry.ID = fmt.Sprintf("history-%d-%s", time.Now().UnixNano()/int64(time.Millisecond), hex.EncodeToString(b))
	}
	if entry.CreatedAt == "" {
		entry.CreatedAt = time.Now().UTC().Format(time.RFC3339)
	}

	if err := repo.Save(entry); err != nil {
		return entry, fmt.Errorf("failed to save entry: %w", err)
	}

	return entry, nil
}

func (g *HistoryGateway) ClearHistory() error {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Println("[HistoryGateway] ClearHistory called")
	repo, err := g.getRepo()
	if err != nil {
		return fmt.Errorf("failed to get repository: %w", err)
	}

	return repo.Clear()
}
