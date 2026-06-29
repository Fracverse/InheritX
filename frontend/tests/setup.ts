import "@testing-library/jest-dom";
import { afterAll, afterEach, beforeAll, vi } from "vitest";
import { server } from "./mocks/server";

// ✅ React 18 fix
globalThis.IS_REACT_ACT_ENVIRONMENT = true;

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

beforeAll(() => {
  server.listen({ onUnhandledRequest: "error" });
});

afterEach(() => {
  vi.clearAllMocks();
  vi.clearAllTimers();
  vi.useRealTimers(); // ✅ important reset per test
});

afterAll(() => {
  server.close();
});