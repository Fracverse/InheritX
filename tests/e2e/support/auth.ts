import type { Page } from "@playwright/test";
import { adminUser, mockWallet } from "../fixtures/users";

export async function seedConnectedWallet(page: Page) {
  await page.addInitScript((wallet) => {
    window.localStorage.setItem("inheritx_wallet_address", wallet.address);
    window.localStorage.setItem("inheritx_wallet_id", wallet.id);
  }, mockWallet);
}

export async function seedAdminSession(page: Page) {
  await page.addInitScript((admin) => {
    window.localStorage.setItem("adminAuth", JSON.stringify({ email: admin.email }));
  }, adminUser);
}
