import "@testing-library/jest-dom";
import { afterAll, afterEach, beforeAll, vi } from "vitest";
import { server } from "./mocks/server";

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

vi.mock("@allbridge/bridge-core-sdk", () => ({
  AllbridgeCoreSdk: vi.fn().mockImplementation(() => ({
    chainDetailsMap: Promise.resolve({}),
    getAmountToBeReceived: vi.fn().mockResolvedValue("95"),
    getGasFeeOptions: vi.fn().mockResolvedValue([{ value: "1.5" }]),
    getAverageTransferTime: vi.fn().mockResolvedValue(900),
    getTransferStatus: vi.fn().mockResolvedValue({ destinationTransactionId: "dest-tx" }),
    bridge: {
      rawTxBuilder: {
        send: vi.fn().mockResolvedValue({ txId: "0xabc", transferId: "transfer-1" }),
      },
    },
  })),
  ChainSymbol: {
    ETH: "ETH",
    BSC: "BSC",
    POL: "POL",
    ARB: "ARB",
    OPT: "OPT",
    AVA: "AVA",
    SOL: "SOL",
    SRB: "SRB",
  },
  Messenger: { ALLBRIDGE: "ALLBRIDGE" },
  nodeRpcUrlsDefault: {},
}));

// Start MSW server before all tests
beforeAll(() => server.listen({ onUnhandledRequest: "error" }));

// Reset handlers after each test
afterEach(() => server.resetHandlers());

// Clean up after all tests
afterAll(() => server.close());
