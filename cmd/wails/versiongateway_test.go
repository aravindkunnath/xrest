package main

import (
	"path/filepath"
	"testing"
	"xrest/internal/models"
)

func TestVersionGateway_Functional(t *testing.T) {
	tempDir := t.TempDir()
	dbPath := filepath.Join(tempDir, "gateway_versions.db")

	vg := NewVersionGateway()
	vg.dbPath = dbPath
	defer func() {
		if vg.repo != nil {
			_ = vg.repo.Close()
		}
	}()

	// 1. Initial state: no versions
	versions, err := vg.GetEndpointVersions("s-1", "e-1", 50)
	if err != nil {
		t.Fatalf("expected no error loading versions, got %v", err)
	}
	if len(versions) != 0 {
		t.Errorf("expected 0 versions, got %d", len(versions))
	}

	// 2. Add versions; version auto-increments
	cfg1 := models.RequestConfig{Method: "GET", URL: "/items"}
	cfg2 := models.RequestConfig{Method: "GET", URL: "/items?page=2"}

	v1, err := vg.AddEndpointVersion("s-1", "e-1", cfg1, 50)
	if err != nil {
		t.Fatalf("expected no error adding version, got %v", err)
	}
	if v1.Version != 1 {
		t.Errorf("expected first version to be 1, got %d", v1.Version)
	}
	v2, err := vg.AddEndpointVersion("s-1", "e-1", cfg2, 50)
	if err != nil {
		t.Fatalf("expected no error adding version, got %v", err)
	}
	if v2.Version != 2 {
		t.Errorf("expected second version to be 2, got %d", v2.Version)
	}

	// 3. Retrieve and verify DESC ordering + config
	versions, err = vg.GetEndpointVersions("s-1", "e-1", 50)
	if err != nil {
		t.Fatalf("expected no error loading versions, got %v", err)
	}
	if len(versions) != 2 {
		t.Fatalf("expected 2 versions, got %d", len(versions))
	}
	if versions[0].Version != 2 || versions[0].Config.URL != "/items?page=2" {
		t.Errorf("expected newest version first with cfg2, got %+v", versions[0])
	}

	// 4. FIFO prune via limit
	vg2 := NewVersionGateway()
	vg2.dbPath = dbPath
	defer func() {
		if vg2.repo != nil {
			_ = vg2.repo.Close()
		}
	}()
	for i := 0; i < 3; i++ {
		_, _ = vg2.AddEndpointVersion("s-1", "e-1", models.RequestConfig{Method: "GET", URL: "/items"}, 3)
	}
	versions, err = vg2.GetEndpointVersions("s-1", "e-1", 10)
	if err != nil {
		t.Fatalf("expected no error loading versions, got %v", err)
	}
	if len(versions) != 3 {
		t.Fatalf("expected 3 versions after prune, got %d", len(versions))
	}
	if versions[0].Version != 5 || versions[2].Version != 3 {
		t.Errorf("expected retained versions 5,4,3, got [%d,%d,%d]",
			versions[0].Version, versions[1].Version, versions[2].Version)
	}

	// 5. Clear only targets the endpoint
	if err := vg2.ClearEndpointVersions("s-1", "e-1"); err != nil {
		t.Fatalf("expected no error clearing versions, got %v", err)
	}
	versions, err = vg2.GetEndpointVersions("s-1", "e-1", 10)
	if err != nil {
		t.Fatalf("expected no error loading versions after clear, got %v", err)
	}
	if len(versions) != 0 {
		t.Errorf("expected 0 versions after clear, got %d", len(versions))
	}
}
