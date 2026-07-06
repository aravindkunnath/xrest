import { test, expect } from "@playwright/test";

test.describe("Request Workspace E2E Suite", () => {
  test.beforeEach(async ({ page }) => {
    // Intercept native Wails Dialogs.Question calls to /wails/runtime
    await page.route("**/wails/runtime", async (route) => {
      const request = route.request();
      if (request.method() === "POST") {
        const payload = request.postDataJSON();
        // ObjectID 5 is Dialog, Method 3 is DialogQuestion
        if (payload && payload.object === 5 && payload.method === 3) {
          await route.fulfill({
            status: 200,
            contentType: "application/json",
            body: JSON.stringify("Yes"), // Simulate clicking "Yes" on the dialog
          });
          return;
        }
      }
      await route.continue();
    });

    // Navigate to the root, clear localStorage, and reload to ensure a clean state
    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
    await page.goto("/");
  });

  test("should create default tab and load correctly", async ({ page }) => {
    // Assert primary layout inputs and buttons are loaded
    await expect(page.locator('button:has-text("Send")')).toBeVisible();
    await expect(
      page.getByPlaceholder("Enter URL"),
    ).toBeVisible();

    // Default tab check
    await expect(
      page.locator('[role="tab"]:has-text("New Request")'),
    ).toBeVisible();
  });

  test("should handle tab lifecycle (create & close)", async ({ page }) => {
    // Click the "New Request" button
    await page.click('button[title^="New Request"]');

    // Verify two tabs exist
    await expect(
      page.locator('[role="tab"]:has-text("New Request")'),
    ).toHaveCount(2);

    // Enter details in the second tab to make it modified
    await page.fill(
      '[role="tabpanel"][data-state="active"] input[placeholder="Enter URL"]',
      "https://api.github.com",
    );

    // Check that the tab title has updated to show the url path
    await expect(
      page.locator('[role="tab"]:has-text("api.github.com")'),
    ).toBeVisible();

    // Close the second tab (triggers Dialog.Question mock returning "Yes")
    await page
      .locator('[role="tab"]:has-text("api.github.com") button')
      .click();

    // Tab count should decrease back to 1 (only counting main workspace tabs, not the sub-tabs)
    const mainTabList = page.getByRole("tablist").first();
    await expect(mainTabList.getByRole("tab")).toHaveCount(1);
  });

  test("should execute mock requests and display outputs", async ({ page }) => {
    // Configure mock request url
    await page.fill(
      '[role="tabpanel"][data-state="active"] input[placeholder="Enter URL"]',
      "https://jsonplaceholder.typicode.com/todos/1",
    );

    // Select Send button
    const sendButton = page.locator('button:has-text("Send")');
    await sendButton.click();

    // Wait for the response details to display (e.g. status code 200)
    await expect(page.locator("text=200 OK")).toBeVisible({ timeout: 10000 });

    // Verify response body contains data
    const responseViewer = page.locator("pre");
    await expect(responseViewer).toContainText('"userId": 1');
  });
});
