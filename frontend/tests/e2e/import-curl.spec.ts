import { test, expect } from "@playwright/test";

test.describe("Import from cURL E2E Suite", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.evaluate(() => {
      localStorage.clear();
      const mockServices = [
        {
          id: "curl-service",
          name: "cURL Target Service",
          directory: "/mock/curl/dir",
          isAuthenticated: false,
          endpoints: [],
          environments: [],
        },
      ];
      localStorage.setItem("mock_services", JSON.stringify(mockServices));
    });
    await page.goto("/");
  });

  test("should open cURL import dialog, paste a command, and import an endpoint", async ({ page }) => {
    // Wait for services to load and render in the explorer
    await expect(page.locator("text=cURL Target Service")).toBeVisible({
      timeout: 10000,
    });

    // Open the Import popover via the Download icon button
    const importTrigger = page.locator('button[title="Import options"]');
    await importTrigger.click();

    // Click the "cURL Command" option
    await page.locator('button:has-text("cURL Command")').click();

    // Dialog should be visible
    await expect(page.locator("text=Import from cURL")).toBeVisible();

    // Select the target service from the dropdown
    const serviceSelect = page.locator("#service-select");
    await serviceSelect.click();
    await page.locator('[role="option"]:has-text("cURL Target Service")').click();

    // Paste a cURL command
    const curlCommand = "curl -X GET https://api.example.com/users";
    await page.fill("#curl-command", curlCommand);

    // The Import Endpoint button should now be enabled
    const importBtn = page.locator('button:has-text("Import Endpoint")');
    await expect(importBtn).toBeEnabled();

    // Click import
    await importBtn.click();

    // The dialog should close on success (mock backend returns the service)
    await expect(page.locator("text=Import from cURL")).not.toBeVisible({
      timeout: 10000,
    });
  });
});
