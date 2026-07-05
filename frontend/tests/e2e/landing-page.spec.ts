import { test, expect } from "@playwright/test";

test("should load the landing page and show the header", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("banner")).toBeVisible();
});
