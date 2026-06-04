import { expect, test } from "@playwright/test";
import { planFixture } from "./fixtures/users";
import { seedConnectedWallet } from "./support/auth";

test.beforeEach(async ({ page }) => {
  await seedConnectedWallet(page);
});

test("creates a plan through the multi-step plan modal", async ({ page }) => {
  await page.goto("/asset-owner/plans");

  await expect(page.getByRole("heading", { name: "Your Plans" })).toBeVisible();
  await page.getByRole("button", { name: /create new plan/i }).click();
  await expect(page.getByRole("heading", { name: "Create Future Plan" })).toBeVisible();

  await page.getByPlaceholder(/wedding fund/i).fill(planFixture.name);
  await page.getByPlaceholder(/describe your plan/i).fill(planFixture.description);
  await page.getByPlaceholder("0.00").fill(planFixture.amount);
  await page.locator('input[type="date"]').fill(planFixture.transferDate);
  await page.getByRole("button", { name: "Next", exact: true }).click();

  await expect(page.getByText("Beneficiary 1")).toBeVisible();
  await page.getByPlaceholder("Full Name").fill(planFixture.beneficiary.name);
  await page.getByPlaceholder("Email").fill(planFixture.beneficiary.email);
  await page.getByPlaceholder("Relationship").fill(planFixture.beneficiary.relationship);
  await page.getByPlaceholder("Allocation").fill(planFixture.beneficiary.allocation);
  await page.getByRole("button", { name: "Next", exact: true }).click();

  await expect(page.getByText(planFixture.name)).toBeVisible();
  await expect(page.getByText(`${planFixture.amount} ETH`)).toBeVisible();
  await page.getByRole("button", { name: "Create Plan" }).click();

  await expect(page.getByRole("heading", { name: "Ready to Approve Tokens" })).toBeVisible();
  await page.getByRole("button", { name: "Start Transaction" }).click();

  await expect(page.getByRole("heading", { name: "Creating Plan..." })).toBeVisible();
  await expect(page.getByText("Waiting for transaction confirmation...")).toBeVisible();
});
