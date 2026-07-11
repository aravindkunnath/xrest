import { test, expect } from "@playwright/test";

test.describe("History E2E Suite", () => {
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

    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
    await page.goto("/");
  });

  test("should record request history, search, filter, and clear history successfully", async ({ page }) => {
    // 1. Send first request (GET /todos/1)
    await page.fill(
      '[role="tabpanel"][data-state="active"] input[placeholder="Enter URL"]',
      "https://jsonplaceholder.typicode.com/todos/1",
    );
    await page.click('button:has-text("Send")');
    await expect(page.locator("text=200 OK")).toBeVisible({ timeout: 10000 });

    // 2. Send second request (GET /todos/2)
    await page.fill(
      '[role="tabpanel"][data-state="active"] input[placeholder="Enter URL"]',
      "https://jsonplaceholder.typicode.com/todos/2",
    );
    await page.click('button:has-text("Send")');
    await expect(page.locator("text=200 OK")).toBeVisible({ timeout: 10000 });

    // 3. Navigate to History View
    const historyNavLink = page.locator('a[href="/history"]');
    if (await historyNavLink.isVisible()) {
      await historyNavLink.click();
    } else {
      await page.goto("/history");
    }

    await expect(page.locator("h1:has-text('History')")).toBeVisible();

    // 4. Verify both entries exist
    await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/1")).toBeVisible();
    await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/2")).toBeVisible();

    // 5. Test Search filtering
    const searchInput = page.locator('input[placeholder="Search URL, headers, or body..."]');
    await searchInput.fill("/todos/2");
    await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/1")).not.toBeVisible();
    await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/2")).toBeVisible();

    // Reset search
    await searchInput.fill("");
    await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/1")).toBeVisible();

    // 6. Test Method filtering
    const historyMethodFilter = page.locator('select').nth(2); // The 3rd select on HistoryView is Method
    if (await historyMethodFilter.isVisible()) {
      await historyMethodFilter.selectOption("GET");
      await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/1")).toBeVisible();
      await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/2")).toBeVisible();
      
      // Select POST (should yield no results matching filters)
      await historyMethodFilter.selectOption("POST");
      await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/1")).not.toBeVisible();
      await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/2")).not.toBeVisible();
      await expect(page.locator("text=No results matching your filters.")).toBeVisible();
      
      // Reset Method filter
      await historyMethodFilter.selectOption("all");
      await expect(page.locator("text=https://jsonplaceholder.typicode.com/todos/1")).toBeVisible();
    }

    // 7. Clear History
    const clearBtn = page.locator('button:has-text("Clear All")');
    await expect(clearBtn).toBeEnabled();
    await clearBtn.click();

    // Verify empty state is displayed
    await expect(page.locator("text=No history yet. Start making requests!")).toBeVisible();
  });
});
