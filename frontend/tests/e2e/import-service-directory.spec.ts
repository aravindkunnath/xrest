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
    await page.evaluate(() => {
      localStorage.clear();
      const mockServices = [
        {
          id: "dir-service",
          name: "Directory Target Service",
          directory: "/mock/dir/service",
          isAuthenticated: false,
          endpoints: [],
          environments: [],
        },
      ];
      localStorage.setItem("mock_services", JSON.stringify(mockServices));
    });
    await page.goto("/");
  });

  test("should trigger directory import from Add Service dialog", async ({ page }) => {
    // Click Add Service button
    const addServiceBtn = page.locator('button[title="Add Service"]');
    await addServiceBtn.click();

    // Verify Add New Service dialog opens
    await expect(page.locator("text=Add New Service")).toBeVisible();

    // Fill service name
    await page.fill("#service-name", "Imported Service");

    // Click folder icon button next to service-dir to select directory (intercepted by Wails mock returning /mock/imported-service-dir)
    await page.click('#service-dir + button');

    // Verify directory path is populated
    await expect(page.locator("#service-dir")).toHaveValue("/mock/imported-service-dir");

    // Click Next
    await page.click('button:has-text("Next")');

    // Click Create Service
    await page.click('button:has-text("Create Service")');

    // Verify success toast
    await expect(page.locator("text=Service Created")).toBeVisible({ timeout: 10000 });
  });
});
