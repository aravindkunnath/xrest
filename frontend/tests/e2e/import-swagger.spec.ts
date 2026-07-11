import { test, expect } from "@playwright/test";

test.describe("Import from Swagger/OpenAPI E2E Suite", () => {
  test.beforeEach(async ({ page }) => {
    // Intercept Wails runtime calls. Dialog.OpenFile is a Dialog (object 5)
    // method; return a canned path for file/directory pickers. Dialog.Question
    // (object 5, method 3) returns "Yes".
    await page.route("**/wails/runtime", async (route) => {
      const request = route.request();
      if (request.method() === "POST") {
        const payload = request.postDataJSON();
        if (payload && payload.object === 5 && payload.method === 3) {
          await route.fulfill({
            status: 200,
            contentType: "application/json",
            body: JSON.stringify("Yes"),
          });
          return;
        }
        // Treat any other Dialog call (e.g. OpenFile) as returning a path
        if (payload && payload.object === 5) {
          await route.fulfill({
            status: 200,
            contentType: "application/json",
            body: JSON.stringify("/mock/path/swagger.json"),
          });
          return;
        }
      }
      await route.continue();
    });

    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
    await page.goto("/");
  });

  test("should open Swagger import dialog and validate required fields", async ({ page }) => {
    // Open the Import popover
    const importTrigger = page.locator('button[title="Import options"]');
    await importTrigger.click();

    // Click the "Swagger / OpenAPI" option
    await page.locator('button:has-text("Swagger / OpenAPI")').click();

    // Dialog should be visible
    await expect(page.locator("text=Import from Swagger/OpenAPI")).toBeVisible();

    const importBtn = page.locator('button:has-text("Import Service")');
    // Import button is disabled until name + directory + (url | file) are provided
    await expect(importBtn).toBeDisabled();

    // Provide a service name
    await page.fill("#swagger-name", "My Swagger API");

    // Still disabled (no directory / source)
    await expect(importBtn).toBeDisabled();

    // Provide a Swagger URL
    await page.fill("#swagger-url", "https://api.example.com/swagger.json");

    // Still disabled without a save directory
    await expect(importBtn).toBeDisabled();

    // Select a save directory (intercepted to return a canned path)
    const dirButton = page
      .locator('button:has-text("Browse")')
      .first();
    // The directory picker is the Folder icon button next to the directory input
    const dirPicker = page.locator('#swagger-dir').locator('..').locator('button').first();
    await dirPicker.click();

    // Now the directory field should be populated by the intercepted picker
    await expect(page.locator('#swagger-dir')).not.toHaveValue("");

    // Import button should now be enabled
    await expect(importBtn).toBeEnabled();
  });
});
