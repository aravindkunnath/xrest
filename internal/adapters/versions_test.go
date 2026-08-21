package adapters

import (
	"path/filepath"
	"testing"
	"xrest/internal/models"
)

func newVersionRepo(t *testing.T) *SqliteVersionRepository {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "versions.db")
	repo, err := NewSqliteVersionRepository(dbPath)
	if err != nil {
		t.Fatalf("failed to create repo: %v", err)
	}
	t.Cleanup(func() { _ = repo.Close() })
	return repo
}

func testVersionConfig(n int) models.RequestConfig {
	return models.RequestConfig{
		Method: "GET",
		URL:    "https://api.example.com/v1/items",
		Params: []models.Param{
			{Name: "page", Value: "1", Enabled: true, Type: "plain"},
		},
		Headers: []models.Header{
			{Name: "X-Request", Value: "test", Enabled: true, Type: "plain"},
		},
		Body: "{\"hello\":\"world\"}",
		Preflight: &models.PreflightConfig{
			Request: &models.Request{
				Method: "POST",
				URL:    "https://auth.example.com/token",
				Headers: map[string]string{
					"Content-Type": "application/json",
				},
			},
			TokenPath:   "access_token",
			TokenHeader: "Authorization",
		},
		Authenticated: true,
		AuthType:      "bearer",
	}
}

func TestSqliteVersionRepository_AddAndGet_DESC(t *testing.T) {
	repo := newVersionRepo(t)

	// GetVersions on empty repo returns no entries
	empty, err := repo.GetVersions("end-1", 50)
	if err != nil {
		t.Fatalf("failed to get versions: %v", err)
	}
	if len(empty) != 0 {
		t.Errorf("expected 0 versions, got %d", len(empty))
	}

	for i := 0; i < 3; i++ {
		if _, err := repo.AddVersion("end-1", testVersionConfig(i), uint64(1000+i), 50); err != nil {
			t.Fatalf("failed to add version %d: %v", i, err)
		}
	}

	versions, err := repo.GetVersions("end-1", 50)
	if err != nil {
		t.Fatalf("failed to get versions: %v", err)
	}
	if len(versions) != 3 {
		t.Fatalf("expected 3 versions, got %d", len(versions))
	}
	if versions[0].Version != 3 || versions[1].Version != 2 || versions[2].Version != 1 {
		t.Errorf("expected DESC ordering [3,2,1], got [%d,%d,%d]",
			versions[0].Version, versions[1].Version, versions[2].Version)
	}
}

func TestSqliteVersionRepository_AutoIncrement(t *testing.T) {
	repo := newVersionRepo(t)

	v1, err := repo.AddVersion("end-1", testVersionConfig(1), 1001, 50)
	if err != nil {
		t.Fatalf("failed to add first version: %v", err)
	}
	if v1.Version != 1 {
		t.Errorf("expected first version to be 1, got %d", v1.Version)
	}

	v2, err := repo.AddVersion("end-1", testVersionConfig(2), 1002, 50)
	if err != nil {
		t.Fatalf("failed to add second version: %v", err)
	}
	if v2.Version != 2 {
		t.Errorf("expected second version to be 2, got %d", v2.Version)
	}

	// A different endpoint restarts the sequence at 1
	other, err := repo.AddVersion("end-2", testVersionConfig(1), 1003, 50)
	if err != nil {
		t.Fatalf("failed to add version to other endpoint: %v", err)
	}
	if other.Version != 1 {
		t.Errorf("expected other endpoint first version to be 1, got %d", other.Version)
	}
}

func TestSqliteVersionRepository_FIFOPrune(t *testing.T) {
	repo := newVersionRepo(t)

	limit := 3
	for i := 0; i < 6; i++ {
		if _, err := repo.AddVersion("end-1", testVersionConfig(i), uint64(1000+i), limit); err != nil {
			t.Fatalf("failed to add version %d: %v", i, err)
		}
	}

	versions, err := repo.GetVersions("end-1", 50)
	if err != nil {
		t.Fatalf("failed to get versions: %v", err)
	}
	if len(versions) != limit {
		t.Fatalf("expected %d versions after prune, got %d", limit, len(versions))
	}

	// Newest `limit` remain: versions 4,5,6 (1..3 pruned away)
	got := []int32{versions[0].Version, versions[1].Version, versions[2].Version}
	want := []int32{6, 5, 4}
	for i := range want {
		if got[i] != want[i] {
			t.Errorf("expected retained versions %v, got %v", want, got)
		}
	}
}

func TestSqliteVersionRepository_JSONRoundTrip(t *testing.T) {
	repo := newVersionRepo(t)

	want := testVersionConfig(1)
	created, err := repo.AddVersion("end-1", want, 1001, 50)
	if err != nil {
		t.Fatalf("failed to add version: %v", err)
	}
	if created.Config.Method != want.Method {
		t.Errorf("expected config method %s, got %s", want.Method, created.Config.Method)
	}
	if created.Config.Preflight == nil || created.Config.Preflight.TokenPath != want.Preflight.TokenPath {
		t.Errorf("preflight did not survive round trip: %+v", created.Config.Preflight)
	}
	if len(created.Config.Params) != 1 || created.Config.Params[0].Name != "page" {
		t.Errorf("params did not survive round trip: %+v", created.Config.Params)
	}

	// Reload from a fresh repo instance to force a real DB read
	versions, err := repo.GetVersions("end-1", 50)
	if err != nil {
		t.Fatalf("failed to get versions: %v", err)
	}
	if len(versions) != 1 {
		t.Fatalf("expected 1 version, got %d", len(versions))
	}
	loaded := versions[0]
	if loaded.Config.Body != want.Body {
		t.Errorf("expected body %q, got %q", want.Body, loaded.Config.Body)
	}
	if loaded.Config.Preflight == nil {
		t.Fatal("expected preflight config to be restored")
	}
	if loaded.Config.Preflight.Request == nil || loaded.Config.Preflight.Request.URL != want.Preflight.Request.URL {
		t.Errorf("preflight request URL not restored: %+v", loaded.Config.Preflight.Request)
	}
	if loaded.Config.Authenticated != want.Authenticated || loaded.Config.AuthType != want.AuthType {
		t.Errorf("auth fields not restored: %+v", loaded.Config)
	}
	if loaded.Config.Preflight.Request.Headers["Content-Type"] != "application/json" {
		t.Errorf("preflight headers not restored: %+v", loaded.Config.Preflight.Request.Headers)
	}
	if loaded.LastUpdated != 1001 {
		t.Errorf("expected lastUpdated 1001, got %d", loaded.LastUpdated)
	}
}

func TestSqliteVersionRepository_CountVersions(t *testing.T) {
	repo := newVersionRepo(t)

	if n, err := repo.CountVersions("end-1"); err != nil || n != 0 {
		t.Fatalf("expected 0 count, got %d (err=%v)", n, err)
	}
	for i := 0; i < 4; i++ {
		if _, err := repo.AddVersion("end-1", testVersionConfig(i), uint64(1000+i), 50); err != nil {
			t.Fatalf("failed to add version: %v", err)
		}
	}
	if _, err := repo.AddVersion("end-2", testVersionConfig(0), 2000, 50); err != nil {
		t.Fatalf("failed to add version to other endpoint: %v", err)
	}

	if n, err := repo.CountVersions("end-1"); err != nil || n != 4 {
		t.Errorf("expected count 4 for end-1, got %d (err=%v)", n, err)
	}
	if n, err := repo.CountVersions("end-2"); err != nil || n != 1 {
		t.Errorf("expected count 1 for end-2, got %d (err=%v)", n, err)
	}
}

func TestSqliteVersionRepository_ClearVersions_OnlyTarget(t *testing.T) {
	repo := newVersionRepo(t)

	for i := 0; i < 3; i++ {
		if _, err := repo.AddVersion("end-1", testVersionConfig(i), uint64(1000+i), 50); err != nil {
			t.Fatalf("failed to add version: %v", err)
		}
	}
	if _, err := repo.AddVersion("end-2", testVersionConfig(0), 2000, 50); err != nil {
		t.Fatalf("failed to add version to other endpoint: %v", err)
	}

	if err := repo.ClearVersions("end-1"); err != nil {
		t.Fatalf("failed to clear versions: %v", err)
	}

	if n, err := repo.CountVersions("end-1"); err != nil || n != 0 {
		t.Errorf("expected 0 versions for end-1 after clear, got %d (err=%v)", n, err)
	}
	if n, err := repo.CountVersions("end-2"); err != nil || n != 1 {
		t.Errorf("expected end-2 to be untouched, got %d (err=%v)", n, err)
	}
}

func TestSqliteVersionRepository_MigrateLegacy(t *testing.T) {
	repo := newVersionRepo(t)

	legacy := []models.EndpointVersion{
		{
			Version:     1,
			Config:      testVersionConfig(1),
			LastUpdated: 1001,
		},
		{
			Version:     2,
			Config:      testVersionConfig(2),
			LastUpdated: 1002,
		},
	}
	if err := repo.MigrateLegacy("end-legacy", legacy); err != nil {
		t.Fatalf("failed to migrate legacy versions: %v", err)
	}

	versions, err := repo.GetVersions("end-legacy", 50)
	if err != nil {
		t.Fatalf("failed to get migrated versions: %v", err)
	}
	if len(versions) != 2 {
		t.Fatalf("expected 2 migrated versions, got %d", len(versions))
	}
	if versions[0].Version != 2 || versions[1].Version != 1 {
		t.Errorf("expected preserved numbering [2,1], got [%d,%d]",
			versions[0].Version, versions[1].Version)
	}
	if versions[0].Config.URL != testVersionConfig(2).URL {
		t.Errorf("config was not preserved for version 2: %+v", versions[0].Config)
	}

	// Re-running migration must not create duplicates (idempotent upsert)
	if err := repo.MigrateLegacy("end-legacy", legacy); err != nil {
		t.Fatalf("failed to re-migrate legacy versions: %v", err)
	}
	if n, err := repo.CountVersions("end-legacy"); err != nil || n != 2 {
		t.Errorf("expected 2 versions after re-migration, got %d (err=%v)", n, err)
	}
}
