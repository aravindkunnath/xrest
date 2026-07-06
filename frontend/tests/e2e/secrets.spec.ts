import { test, expect } from "@playwright/test";

test.describe("Secrets Management E2E Suite", () => {
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

    // Set XREST_ENV to test to enable the in-memory keyring backend
    await page.goto("/");
    
    // Navigate to Secrets Management View
    const secretsNavLink = page.locator('a[href="/secrets"]');
    if (await secretsNavLink.isVisible()) {
      await secretsNavLink.click();
    } else {
      await page.goto("/secrets");
    }
    
    await expect(page.locator("h1:has-text('Secrets Management')")).toBeVisible();
  });

  test("should add, reveal, and delete secrets successfully", async ({ page }) => {
    // 1. Initial State: Check for empty state elements
    const emptyState = page.locator("text=No secrets found");
    
    // 2. Add Secret
    // Click either the main action button or the empty state action button
    const addSecretBtn = page.locator('button:has-text("Add Secret"), button:has-text("Add your first secret")').first();
    await addSecretBtn.click();

    // Verify dialog appears
    await expect(page.locator("text=Add New Secret")).toBeVisible();

    const testKey = "PLAYWRIGHT_TEST_KEY";
    const testVal = "super-secret-playwright-token";

    await page.fill('input[placeholder="Secret Name (e.g. OPENAI_API_KEY)"]', testKey);
    await page.fill('input[placeholder="Secret Value"]', testVal);

    // Save the secret
    await page.click('button:has-text("Save Secret")');

    // Dialog should dismiss and table row with the key should show up
    await expect(page.locator(`text=${testKey}`)).toBeVisible();
    await expect(page.locator("text=••••••••••••")).toBeVisible();

    // 3. Reveal Secret
    // Identify the row matching the key
    const row = page.locator(`tr:has-text("${testKey}")`);
    const revealBtn = row.locator('button:has(svg.lucide-eye)');
    await revealBtn.click();

    // Verify secret content is decrypted and displayed
    await expect(row.locator(`text=${testVal}`)).toBeVisible();

    // 4. Delete Secret
    const deleteBtn = row.locator('button:has(svg.lucide-trash-2)');
    await deleteBtn.click();

    // Key should disappear from the table list
    await expect(page.locator(`text=${testKey}`)).not.toBeVisible();
  });
});