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
    // Hover over the service entry to make Settings button visible
    const serviceNode = page.locator("text=Git Integrated Service");
    await serviceNode.hover();

    // Click on Service Settings button
    const settingsButton = page.locator('button[title="Service Settings"]');
    await expect(settingsButton).toBeVisible();
    await settingsButton.click();

    // Verify Settings tab is loaded
    await expect(
      page.locator('[role="tab"]:has-text("Git Integrated Service")')
    ).toBeVisible();

    // Verify that the Git Status section and details are displayed.
    await expect(page.locator("text=Git Status")).toBeVisible();
    await expect(page.getByText("Branch", { exact: true })).toBeVisible();
    await expect(page.getByText("Remote", { exact: true })).toBeVisible();

    // The mock returns hasUncommittedChanges: false. Verify stage shows "Clean"
    await expect(page.locator("text=Clean")).toBeVisible();

    // Commit button should be disabled when there are no uncommitted changes
    const commitButton = page.locator('button:has-text("Commit")');
    await expect(commitButton).toBeDisabled();
  });
});
