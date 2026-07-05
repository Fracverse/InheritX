import type { EvmProvider, OriginChainId, SolanaProvider, SourceWalletType } from "./types";
import { getOriginChain } from "./chains";

export function getEvmProvider(): EvmProvider | undefined {
  if (typeof window === "undefined") return undefined;
  return window.ethereum;
}

export function getSolanaProvider(): SolanaProvider | undefined {
  if (typeof window === "undefined") return undefined;
  return window.phantom?.solana ?? window.solana;
}

export async function connectEvmWallet(): Promise<string> {
  const provider = getEvmProvider();
  if (!provider) {
    throw new Error("MetaMask or another EVM wallet is not installed.");
  }

  const accounts = (await provider.request({
    method: "eth_requestAccounts",
  })) as string[];

  const address = accounts[0];
  if (!address) {
    throw new Error("No EVM account returned from wallet.");
  }
  return address;
}

export async function connectSolanaWallet(): Promise<string> {
  const provider = getSolanaProvider();
  if (!provider) {
    throw new Error("Phantom or another Solana wallet is not installed.");
  }

  const response = await provider.connect();
  return response.publicKey.toString();
}

export async function switchEvmChain(chainId: OriginChainId): Promise<void> {
  const provider = getEvmProvider();
  if (!provider) return;

  const hexChainId = EVM_CHAIN_IDS[chainId];
  if (!hexChainId) return;

  try {
    await provider.request({
      method: "wallet_switchEthereumChain",
      params: [{ chainId: hexChainId }],
    });
  } catch (error) {
    const err = error as { code?: number };
    if (err.code === 4902) {
      await provider.request({
        method: "wallet_addEthereumChain",
        params: [EVM_CHAIN_PARAMS[chainId]],
      });
    } else {
      throw error;
    }
  }
}

export function walletTypeForChain(chainId: OriginChainId): SourceWalletType {
  return getOriginChain(chainId)?.walletType ?? "evm";
}

const EVM_CHAIN_IDS: Partial<Record<OriginChainId, string>> = {
  ETH: "0x1",
  BSC: "0x38",
  POL: "0x89",
  ARB: "0xa4b1",
  OPT: "0xa",
  AVA: "0xa86a",
};

const EVM_CHAIN_PARAMS: Partial<
  Record<
    OriginChainId,
    {
      chainId: string;
      chainName: string;
      nativeCurrency: { name: string; symbol: string; decimals: number };
      rpcUrls: string[];
      blockExplorerUrls: string[];
    }
  >
> = {
  ETH: {
    chainId: "0x1",
    chainName: "Ethereum Mainnet",
    nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
    rpcUrls: ["https://ethereum.publicnode.com"],
    blockExplorerUrls: ["https://etherscan.io"],
  },
  BSC: {
    chainId: "0x38",
    chainName: "BNB Smart Chain",
    nativeCurrency: { name: "BNB", symbol: "BNB", decimals: 18 },
    rpcUrls: ["https://bsc-dataseed.binance.org"],
    blockExplorerUrls: ["https://bscscan.com"],
  },
  POL: {
    chainId: "0x89",
    chainName: "Polygon",
    nativeCurrency: { name: "MATIC", symbol: "MATIC", decimals: 18 },
    rpcUrls: ["https://polygon-rpc.com"],
    blockExplorerUrls: ["https://polygonscan.com"],
  },
  ARB: {
    chainId: "0xa4b1",
    chainName: "Arbitrum One",
    nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
    rpcUrls: ["https://arb1.arbitrum.io/rpc"],
    blockExplorerUrls: ["https://arbiscan.io"],
  },
  OPT: {
    chainId: "0xa",
    chainName: "Optimism",
    nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
    rpcUrls: ["https://mainnet.optimism.io"],
    blockExplorerUrls: ["https://optimistic.etherscan.io"],
  },
  AVA: {
    chainId: "0xa86a",
    chainName: "Avalanche C-Chain",
    nativeCurrency: { name: "AVAX", symbol: "AVAX", decimals: 18 },
    rpcUrls: ["https://api.avax.network/ext/bc/C/rpc"],
    blockExplorerUrls: ["https://snowtrace.io"],
  },
};
