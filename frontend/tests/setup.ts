import "@testing-library/jest-dom";
import { afterAll, afterEach, beforeAll, vi } from "vitest";
import { server } from "./mocks/server";

// Mock Stellar Freighter API
vi.mock("@stellar/freighter-api", () => ({
  getAddress: vi.fn(),
  signTransaction: vi.fn(),
  isConnected: vi.fn(),
}));

// Mock Stellar Wallets Kit
vi.mock("@creit.tech/stellar-wallets-kit", () => ({
  StellarWalletsKit: vi.fn().mockImplementation(() => ({
    setWallet: vi.fn(),
    getAddress: vi.fn().mockResolvedValue({ address: "MOCK_ADDRESS" }),
    disconnect: vi.fn(),
  })),
  WalletNetwork: {
    TESTNET: "TESTNET",
    PUBLIC: "PUBLIC",
  },
  allowAllModules: vi.fn().mockReturnValue([]),
}));

// Start MSW server before all tests
beforeAll(() => server.listen({ onUnhandledRequest: "error" }));

// Reset handlers after each test
afterEach(() => server.resetHandlers());

// Clean up after all tests
afterAll(() => server.close());
