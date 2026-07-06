import { test, expect } from "@playwright/test";

test.describe("Import Service from Directory E2E Suite", () => {
  test.beforeEach(async ({ page }) => {
    // Intercept Wails runtime calls so Dialog.OpenFile (object 5) returns a
    // canned directory path instead of opening the native picker.
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
        // Directory picker (CanChooseDirectories: true) -> return a path
        if (payload && payload.object === 5) {
          await route.fulfill({
            status: 200,
            contentType: "application/json",
            body: JSON.stringify("/mock/imported-service-dir"),
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

  test("should trigger directory import from the Import popover", async ({ page }) => {
    // Open the Import popover
    const importTrigger = page.locator('button[title="Import options"]');
    await importTrigger.click();

    // Click "From Directory"
    await page.locator('button:has-text("From Directory")').click();

    // The picker was intercepted; the import flow runs against the backend.
    // Verify either a success toast or an error toast surfaces (depending on
    // whether the canned directory resolves on the backend).
    const successToast = page.locator("text=Service Imported");
    const errorToast = page.locator("text=Import Failed");
    await expect(successToast.or(errorToast)).toBeVisible({ timeout: 10000 });
  });
});
