import { expect, test } from "@playwright/test";
import { claimFixture } from "./fixtures/users";
import { seedConnectedWallet } from "./support/auth";

test.beforeEach(async ({ page }) => {
  await seedConnectedWallet(page);
  await page.addInitScript(() => {
    Math.random = () => 0.99;
  });
});

test("submits a claim and reaches the plan summary", async ({ page }) => {
  await page.goto("/asset-owner/claim");
  await page.waitForLoadState("networkidle");

  await expect(page.getByRole("heading", { name: "Claim Plan" })).toBeVisible();
  await page.getByRole("link", { name: "CLAIM PLAN" }).first().click();
  await expect(
    page.getByPlaceholder("Enter the name of your beneficiary"),
  ).toBeVisible();
  await expect(page.getByTestId("claim-form-hydrated")).toBeAttached();

  await page
    .getByPlaceholder("Enter the name of your beneficiary")
    .fill(claimFixture.beneficiaryName);
  await page
    .getByPlaceholder("Enter the email of your beneficiary")
    .fill(claimFixture.beneficiaryEmail);

  const codeInputs = page.locator('input[id^="claim-code-"]');
  for (const [index, digit] of [...claimFixture.claimCode].entries()) {
    await codeInputs.nth(index).click();
    await page.keyboard.press(digit);
  }

  await page.getByRole("link", { name: /claim inheritance/i }).click();
  await expect(page.getByText("Inheritance claimed is Successful")).toBeVisible();

  await page.getByRole("link", { name: "Continue" }).click();
  await expect(page.getByRole("heading", { name: "Plan Summary" })).toBeVisible();
  await expect(page.getByRole("button", { name: "WITHDRAW ALL" })).toBeVisible();
});
