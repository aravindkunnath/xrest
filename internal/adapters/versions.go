package adapters

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"xrest/internal/models"

	_ "modernc.org/sqlite"
)

// HistoryDbPath returns the shared SQLite database path used for request
// history and endpoint version history. Respects XREST_ENV=test for isolated
// test environments.
func HistoryDbPath() string {
	if os.Getenv("XREST_ENV") == "test" {
		return filepath.Join(os.TempDir(), "xrest-test", "history.db")
	}
	return filepath.Join(os.Getenv("HOME"), ".xrest", "history.db")
}

// SqliteVersionRepository persists endpoint version history in SQLite. It uses
// the same database file as SqliteHistoryRepository (~/.xrest/history.db) and
// adds the endpoint_versions table additively.
type SqliteVersionRepository struct {
	db *sql.DB
}

func NewSqliteVersionRepository(dbPath string) (*SqliteVersionRepository, error) {
	dir := filepath.Dir(dbPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create db directory: %w", err)
	}

	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %w", err)
	}

	repo := &SqliteVersionRepository{db: db}
	if err := repo.Init(); err != nil {
		db.Close()
		return nil, err
	}

	return repo, nil
}

func (r *SqliteVersionRepository) Close() error {
	if r.db != nil {
		return r.db.Close()
	}
	return nil
}

func (r *SqliteVersionRepository) Init() error {
	query := `
	CREATE TABLE IF NOT EXISTS endpoint_versions (
		endpoint_id  TEXT NOT NULL,
		version      INTEGER NOT NULL,
		config_json  TEXT NOT NULL,
		last_updated INTEGER NOT NULL,
		PRIMARY KEY (endpoint_id, version)
	)`
	if _, err := r.db.Exec(query); err != nil {
		return fmt.Errorf("failed to initialize endpoint_versions table: %w", err)
	}
	return nil
}

// AddVersion computes version = MAX(version) + 1 for the endpoint, inserts the
// snapshot, then prunes the oldest versions so only the newest maxVersions
// remain (FIFO overwrite).
func (r *SqliteVersionRepository) AddVersion(endpointID string, config models.RequestConfig, lastUpdated uint64, maxVersions int) (models.EndpointVersion, error) {
	version := int64(0)
	err := r.db.QueryRow(
		`SELECT COALESCE(MAX(version), 0) FROM endpoint_versions WHERE endpoint_id = ?`,
		endpointID,
	).Scan(&version)
	if err != nil {
		return models.EndpointVersion{}, fmt.Errorf("failed to compute next version: %w", err)
	}

	configBytes, err := json.Marshal(config)
	if err != nil {
		return models.EndpointVersion{}, fmt.Errorf("failed to marshal config: %w", err)
	}

	next := version + 1
	_, err = r.db.Exec(
		`INSERT INTO endpoint_versions (endpoint_id, version, config_json, last_updated) VALUES (?, ?, ?, ?)`,
		endpointID, next, string(configBytes), lastUpdated,
	)
	if err != nil {
		return models.EndpointVersion{}, fmt.Errorf("failed to insert endpoint version: %w", err)
	}

	if err := r.prune(endpointID, maxVersions); err != nil {
		return models.EndpointVersion{}, err
	}

	return models.EndpointVersion{
		Version:     int32(next),
		Config:      config,
		LastUpdated: lastUpdated,
	}, nil
}

// prune deletes all but the newest maxVersions for the endpoint (FIFO).
func (r *SqliteVersionRepository) prune(endpointID string, maxVersions int) error {
	if maxVersions <= 0 {
		return fmt.Errorf("maxVersions must be positive")
	}
	_, err := r.db.Exec(
		`DELETE FROM endpoint_versions
		 WHERE endpoint_id = ?
		   AND version <= (
			   SELECT MAX(version) FROM endpoint_versions WHERE endpoint_id = ?
		   ) - ?`,
		endpointID, endpointID, maxVersions,
	)
	if err != nil {
		return fmt.Errorf("failed to prune endpoint versions: %w", err)
	}
	return nil
}

// GetVersions returns the newest `limit` versions ordered by version DESC.
func (r *SqliteVersionRepository) GetVersions(endpointID string, limit int) ([]models.EndpointVersion, error) {
	rows, err := r.db.Query(
		`SELECT version, config_json, last_updated
		 FROM endpoint_versions
		 WHERE endpoint_id = ?
		 ORDER BY version DESC
		 LIMIT ?`,
		endpointID, limit,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to query endpoint versions: %w", err)
	}
	defer rows.Close()

	var versions []models.EndpointVersion
	for rows.Next() {
		var v models.EndpointVersion
		var configJSON string
		if err := rows.Scan(&v.Version, &configJSON, &v.LastUpdated); err != nil {
			return nil, fmt.Errorf("failed to scan endpoint version row: %w", err)
		}
		if err := json.Unmarshal([]byte(configJSON), &v.Config); err != nil {
			return nil, fmt.Errorf("failed to unmarshal endpoint version config: %w", err)
		}
		versions = append(versions, v)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating endpoint version rows: %w", err)
	}

	return versions, nil
}

func (r *SqliteVersionRepository) CountVersions(endpointID string) (int, error) {
	var count int
	if err := r.db.QueryRow(
		`SELECT COUNT(*) FROM endpoint_versions WHERE endpoint_id = ?`,
		endpointID,
	).Scan(&count); err != nil {
		return 0, fmt.Errorf("failed to count endpoint versions: %w", err)
	}
	return count, nil
}

func (r *SqliteVersionRepository) ClearVersions(endpointID string) error {
	if _, err := r.db.Exec(
		`DELETE FROM endpoint_versions WHERE endpoint_id = ?`,
		endpointID,
	); err != nil {
		return fmt.Errorf("failed to clear endpoint versions: %w", err)
	}
	return nil
}

// MigrateLegacy seeds versions previously stored inline in endpoint YAML files
// into SQLite. It is idempotent (INSERT OR IGNORE) so reprocessing an endpoint
// never creates duplicates. Best-effort by design; callers treat failures as
// non-blocking.
func (r *SqliteVersionRepository) MigrateLegacy(endpointID string, versions []models.EndpointVersion) error {
	for _, v := range versions {
		configBytes, err := json.Marshal(v.Config)
		if err != nil {
			return fmt.Errorf("failed to marshal legacy config: %w", err)
		}
		_, err = r.db.Exec(
			`INSERT OR IGNORE INTO endpoint_versions (endpoint_id, version, config_json, last_updated) VALUES (?, ?, ?, ?)`,
			endpointID, v.Version, string(configBytes), v.LastUpdated,
		)
		if err != nil {
			return fmt.Errorf("failed to migrate legacy endpoint version: %w", err)
		}
	}
	return nil
}
