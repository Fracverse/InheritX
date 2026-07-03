import type { OriginChain, OriginChainId, SourceToken } from "./types";

export const DESTINATION_CHAIN = "SRB" as const;

export const ORIGIN_CHAINS: OriginChain[] = [
  { id: "ETH", name: "Ethereum", walletType: "evm" },
  { id: "BSC", name: "BNB Chain", walletType: "evm" },
  { id: "POL", name: "Polygon", walletType: "evm" },
  { id: "ARB", name: "Arbitrum", walletType: "evm" },
  { id: "OPT", name: "Optimism", walletType: "evm" },
  { id: "AVA", name: "Avalanche", walletType: "evm" },
  { id: "SOL", name: "Solana", walletType: "solana" },
];

export const DEFAULT_TOKENS: Record<OriginChainId, SourceToken[]> = {
  ETH: [
    {
      symbol: "USDC",
      name: "USD Coin",
      address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      decimals: 6,
      chainId: "ETH",
    },
    {
      symbol: "USDT",
      name: "Tether USD",
      address: "0xdAC17F958D2ee523a2206206994597C13D831ec7",
      decimals: 6,
      chainId: "ETH",
    },
  ],
  BSC: [
    {
      symbol: "USDC",
      name: "USD Coin",
      address: "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d",
      decimals: 18,
      chainId: "BSC",
    },
    {
      symbol: "USDT",
      name: "Tether USD",
      address: "0x55d398326f99059fF775485246999027B3197955",
      decimals: 18,
      chainId: "BSC",
    },
  ],
  POL: [
    {
      symbol: "USDC",
      name: "USD Coin",
      address: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
      decimals: 6,
      chainId: "POL",
    },
    {
      symbol: "USDT",
      name: "Tether USD",
      address: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
      decimals: 6,
      chainId: "POL",
    },
  ],
  ARB: [
    {
      symbol: "USDC",
      name: "USD Coin",
      address: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
      decimals: 6,
      chainId: "ARB",
    },
  ],
  OPT: [
    {
      symbol: "USDC",
      name: "USD Coin",
      address: "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85",
      decimals: 6,
      chainId: "OPT",
    },
  ],
  AVA: [
    {
      symbol: "USDC",
      name: "USD Coin",
      address: "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E",
      decimals: 6,
      chainId: "AVA",
    },
  ],
  SOL: [
    {
      symbol: "USDC",
      name: "USD Coin",
      address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      decimals: 6,
      chainId: "SOL",
    },
    {
      symbol: "USDT",
      name: "Tether USD",
      address: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
      decimals: 6,
      chainId: "SOL",
    },
  ],
};

export function getOriginChain(chainId: OriginChainId): OriginChain | undefined {
  return ORIGIN_CHAINS.find((c) => c.id === chainId);
}

export function getTokensForChain(chainId: OriginChainId): SourceToken[] {
  return DEFAULT_TOKENS[chainId] ?? [];
}
