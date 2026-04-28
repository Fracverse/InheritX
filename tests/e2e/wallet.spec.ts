import { expect, test } from "@playwright/test";
import { mockWallet } from "./fixtures/users";

test("connects a wallet through the wallet modal", async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => window.localStorage.clear());
  await page.reload();

  await page.getByRole("button", { name: /connect wallet/i }).click();
  await expect(page.getByRole("heading", { name: "Connect Wallet" })).toBeVisible();

  await page.getByRole("button", { name: /freighter/i }).click();
  await page
    .locator("div")
    .filter({ has: page.getByRole("heading", { name: "Connect Wallet" }) })
    .getByRole("button", { name: /^connect wallet$/i })
    .click();

  await expect(page.getByText(/Good morning/i)).toBeVisible();
  await expect(page.getByText(mockWallet.shortAddress)).toBeVisible();
});
