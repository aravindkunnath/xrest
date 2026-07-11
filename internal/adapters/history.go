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

type SqliteHistoryRepository struct {
	db *sql.DB
}

func NewSqliteHistoryRepository(dbPath string) (*SqliteHistoryRepository, error) {
	dir := filepath.Dir(dbPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create db directory: %w", err)
	}

	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %w", err)
	}

	repo := &SqliteHistoryRepository{db: db}
	if err := repo.Init(); err != nil {
		db.Close()
		return nil, err
	}

	return repo, nil
}

func (r *SqliteHistoryRepository) Close() error {
	if r.db != nil {
		return r.db.Close()
	}
	return nil
}

func (r *SqliteHistoryRepository) Init() error {
	query := `
	CREATE TABLE IF NOT EXISTS history (
		id TEXT PRIMARY KEY,
		service_id TEXT,
		endpoint_id TEXT,
		method TEXT NOT NULL,
		url TEXT NOT NULL,
		request_headers TEXT NOT NULL,
		request_body TEXT NOT NULL,
		response_status INTEGER NOT NULL,
		response_status_text TEXT NOT NULL,
		response_headers TEXT NOT NULL,
		response_body TEXT NOT NULL,
		time_elapsed INTEGER NOT NULL,
		size INTEGER NOT NULL,
		created_at TEXT NOT NULL
	)`
	_, err := r.db.Exec(query)
	if err != nil {
		return fmt.Errorf("failed to initialize history table: %w", err)
	}
	return nil
}

func (r *SqliteHistoryRepository) Save(entry models.HistoryEntry) error {
	reqHeadersBytes, err := json.Marshal(entry.RequestHeaders)
	if err != nil {
		return fmt.Errorf("failed to marshal request headers: %w", err)
	}

	respHeadersBytes, err := json.Marshal(entry.ResponseHeaders)
	if err != nil {
		return fmt.Errorf("failed to marshal response headers: %w", err)
	}

	query := `
	INSERT INTO history (
		id, service_id, endpoint_id, method, url,
		request_headers, request_body,
		response_status, response_status_text,
		response_headers, response_body,
		time_elapsed, size, created_at
	) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`

	_, err = r.db.Exec(query,
		entry.ID,
		entry.ServiceID,
		entry.EndpointID,
		entry.Method,
		entry.URL,
		string(reqHeadersBytes),
		entry.RequestBody,
		entry.ResponseStatus,
		entry.ResponseStatusText,
		string(respHeadersBytes),
		entry.ResponseBody,
		entry.TimeElapsed,
		entry.Size,
		entry.CreatedAt,
	)
	if err != nil {
		return fmt.Errorf("failed to insert history entry: %w", err)
	}
	return nil
}

func (r *SqliteHistoryRepository) GetHistory(limit, offset int) ([]models.HistoryEntry, error) {
	query := `
	SELECT
		id, service_id, endpoint_id, method, url,
		request_headers, request_body,
		response_status, response_status_text,
		response_headers, response_body,
		time_elapsed, size, created_at
	FROM history
	ORDER BY created_at DESC
	LIMIT ? OFFSET ?`

	rows, err := r.db.Query(query, limit, offset)
	if err != nil {
		return nil, fmt.Errorf("failed to query history: %w", err)
	}
	defer rows.Close()

	var entries []models.HistoryEntry
	for rows.Next() {
		var entry models.HistoryEntry
		var reqHeadersStr, respHeadersStr string

		err := rows.Scan(
			&entry.ID,
			&entry.ServiceID,
			&entry.EndpointID,
			&entry.Method,
			&entry.URL,
			&reqHeadersStr,
			&entry.RequestBody,
			&entry.ResponseStatus,
			&entry.ResponseStatusText,
			&respHeadersStr,
			&entry.ResponseBody,
			&entry.TimeElapsed,
			&entry.Size,
			&entry.CreatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan history row: %w", err)
		}

		if err := json.Unmarshal([]byte(reqHeadersStr), &entry.RequestHeaders); err != nil {
			entry.RequestHeaders = []models.Header{}
		}
		if err := json.Unmarshal([]byte(respHeadersStr), &entry.ResponseHeaders); err != nil {
			entry.ResponseHeaders = []models.Header{}
		}

		entries = append(entries, entry)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iteration history rows: %w", err)
	}

	return entries, nil
}

func (r *SqliteHistoryRepository) Clear() error {
	_, err := r.db.Exec("DELETE FROM history")
	if err != nil {
		return fmt.Errorf("failed to clear history: %w", err)
	}
	return nil
}
