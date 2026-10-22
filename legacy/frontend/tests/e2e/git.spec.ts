import { test, expect } from "@playwright/test";

test.describe("Git Integration E2E Suite", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to root, clear/seed localStorage, and reload
    await page.goto("/");
    await page.evaluate(() => {
      localStorage.clear();
      const mockServices = [
        {
          id: "git-service",
          name: "Git Integrated Service",
          directory: "/mock/git/dir",
          isAuthenticated: false,
          endpoints: [],
          environments: []
        }
      ];
      localStorage.setItem("mock_services", JSON.stringify(mockServices));
    });
    await page.goto("/");
  });

  test("should display Git integration UI elements and handle actions", async ({ page }) => {
    // Click on Configuration button in sidebar
    const settingsButton = page.locator('button:has-text("Configuration")');
    await expect(settingsButton).toBeVisible();
    await settingsButton.click();

    // Click on Git Status section in settings LHS menu
    const gitSectionBtn = page.locator('button:has-text("Git Status")').first();
    await gitSectionBtn.click();

    // Verify that the Git Status section details are displayed.
    await expect(page.getByText("Branch", { exact: true })).toBeVisible();
    await expect(page.getByText("Remote", { exact: true })).toBeVisible();

    // The mock returns hasUncommittedChanges: false. Verify stage shows "Clean"
    await expect(page.getByText("Clean", { exact: true })).toBeVisible();

    // Commit button should be disabled when there are no uncommitted changes
    const commitButton = page.locator('button:has-text("Commit")');
    await expect(commitButton).toBeDisabled();
  });
});
