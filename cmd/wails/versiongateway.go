package main

import (
	"fmt"
	"log"
	"sync"
	"time"

	"xrest/internal/adapters"
	"xrest/internal/models"
)

// VersionGateway exposes endpoint version history (stored in SQLite) to the
// frontend. serviceId is accepted for introspection/debugging only — versions
// are keyed by endpointId.
type VersionGateway struct {
	mu     sync.Mutex
	repo   *adapters.SqliteVersionRepository
	dbPath string
}

func NewVersionGateway() *VersionGateway {
	return &VersionGateway{
		dbPath: adapters.HistoryDbPath(),
	}
}

func (g *VersionGateway) getRepo() (*adapters.SqliteVersionRepository, error) {
	if g.repo != nil {
		return g.repo, nil
	}
	repo, err := adapters.NewSqliteVersionRepository(g.dbPath)
	if err != nil {
		return nil, err
	}
	g.repo = repo
	return g.repo, nil
}

// GetEndpointVersions returns the newest `limit` versions for an endpoint,
// ordered newest-first.
func (g *VersionGateway) GetEndpointVersions(serviceId, endpointId string, limit int) ([]models.EndpointVersion, error) {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Printf("[VersionGateway] GetEndpointVersions called for endpoint %s", endpointId)
	repo, err := g.getRepo()
	if err != nil {
		return nil, fmt.Errorf("failed to get repository: %w", err)
	}
	return repo.GetVersions(endpointId, limit)
}

// AddEndpointVersion creates a new version (version = MAX+1), stores it, then
// prunes the oldest so only the newest maxVersions remain (FIFO).
func (g *VersionGateway) AddEndpointVersion(serviceId, endpointId string, config models.RequestConfig, maxVersions int) (models.EndpointVersion, error) {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Printf("[VersionGateway] AddEndpointVersion called for endpoint %s, maxVersions %d", endpointId, maxVersions)
	repo, err := g.getRepo()
	if err != nil {
		return models.EndpointVersion{}, fmt.Errorf("failed to get repository: %w", err)
	}
	return repo.AddVersion(endpointId, config, uint64(time.Now().Unix()), maxVersions)
}

// ClearEndpointVersions deletes all versions for an endpoint (used on endpoint
// deletion).
func (g *VersionGateway) ClearEndpointVersions(serviceId, endpointId string) error {
	g.mu.Lock()
	defer g.mu.Unlock()

	log.Printf("[VersionGateway] ClearEndpointVersions called for endpoint %s", endpointId)
	repo, err := g.getRepo()
	if err != nil {
		return fmt.Errorf("failed to get repository: %w", err)
	}
	return repo.ClearVersions(endpointId)
}
