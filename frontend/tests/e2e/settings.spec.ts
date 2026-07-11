import { test, expect } from "@playwright/test";

test.describe("Settings and Theme E2E Suite", () => {
  test.beforeEach(async ({ page }) => {
    // Intercept Wails runtime calls if needed
    await page.route("**/wails/runtime", async (route) => {
      await route.continue();
    });

    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
    await page.goto("/");
    
    // Navigate to Settings View via link
    const settingsNavLink = page.locator('a[href="/settings"]');
    await settingsNavLink.click();
  });

  test("should load settings page correctly", async ({ page }) => {
    await expect(page.locator("h1:has-text('Settings')")).toBeVisible();
    await expect(page.locator("h2:has-text('Appearance')")).toBeVisible();
  });

  test("should allow changing the theme to light/dark", async ({ page }) => {
    // Select the theme dropdown
    const selectTrigger = page.locator("#theme-select");
    await selectTrigger.click();

    // Choose dark option
    const darkOption = page.locator("div[role='option'] >> text=Dark");
    await darkOption.click();

    // Verify localStorage has changed
    const settingsJson = await page.evaluate(() => localStorage.getItem("xrest_settings"));
    expect(settingsJson).not.toBeNull();
    const settings = JSON.parse(settingsJson!);
    expect(settings.theme).toBe("dark");

    // Choose light option
    await selectTrigger.click();
    const lightOption = page.locator("div[role='option'] >> text=Light");
    await lightOption.click();

    // Verify localStorage has changed
    const newSettingsJson = await page.evaluate(() => localStorage.getItem("xrest_settings"));
    const newSettings = JSON.parse(newSettingsJson!);
    expect(newSettings.theme).toBe("light");
  });
});
