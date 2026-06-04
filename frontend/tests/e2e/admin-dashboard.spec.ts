import { expect, test } from "@playwright/test";
import { adminUser } from "./fixtures/users";

test("protects admin dashboard and allows admin login", async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => window.localStorage.clear());

  await page.goto("/admin");
  await expect(page).toHaveURL(/\/admin\/login$/);
  await expect(page.getByRole("heading", { name: "Admin Login" })).toBeVisible();

  await page.getByPlaceholder("admin@inheritx.com").fill(adminUser.email);
  await page.getByPlaceholder("Enter your password").fill(adminUser.password);
  await page.getByRole("button", { name: "Sign In" }).click();

  await expect(page).toHaveURL(/\/admin$/);
  await expect(page.getByRole("heading", { name: /admin dashboard/i })).toBeVisible();
  await expect(page.getByText("Quick Actions")).toBeVisible();
  await expect(page.getByRole("button", { name: /review kyc/i })).toBeVisible();
});
