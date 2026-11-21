'use client';

import React, { useState, useEffect } from 'react';
import { X, Loader, AlertCircle, CheckCircle, Zap } from 'lucide-react';
import clsx from 'clsx';

interface WalletModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConnect: (wallet: string) => Promise<void>;
  currentWallet?: string;
  currentBalance?: number;
  currentNetwork?: string;
}

type NetworkType = 'mainnet' | 'testnet' | 'futurenet' | 'unknown';

interface NetworkInfo {
  passphrase: string;
  name: string;
  rpcUrl: string;
  friendbotUrl?: string;
}

const NETWORKS: Record<NetworkType, NetworkInfo> = {
  mainnet: {
    passphrase: 'Public Global Stellar Network ; September 2015',
    name: 'Mainnet',
    rpcUrl: 'https://rpc-mainnet.stellar.org',
  },
  testnet: {
    passphrase: 'Test SDF Network ; September 2015',
    name: 'Testnet',
    rpcUrl: 'https://soroban-testnet.stellar.org',
    friendbotUrl: 'https://friendbot.stellar.org',
  },
  futurenet: {
    passphrase: 'Test SDF Future Network ; October 2022',
    name: 'Futurenet',
    rpcUrl: 'https://rpc-futurenet.stellar.org',
    friendbotUrl: 'https://friendbot-futurenet.stellar.org',
  },
  unknown: {
    passphrase: 'Unknown',
    name: 'Unknown Network',
    rpcUrl: '',
  },
};

export function WalletModal({
  isOpen,
  onClose,
  onConnect,
  currentWallet,
  currentBalance = 0,
  currentNetwork = 'unknown',
}: WalletModalProps) {
  const [selectedWallet, setSelectedWallet] = useState<string>('');
  const [detectedNetwork, setDetectedNetwork] = useState<NetworkType>('unknown');
  const [isConnecting, setIsConnecting] = useState(false);
  const [isFundingAccount, setIsFundingAccount] = useState(false);
  const [fundingStatus, setFundingStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [fundingMessage, setFundingMessage] = useState('');
  const [showNetworkWarning, setShowNetworkWarning] = useState(false);
  const [lowBalance, setLowBalance] = useState(false);

  const walletOptions = [
    { id: 'freighter', name: 'Freighter Wallet', icon: '🔐' },
    { id: 'albedo', name: 'Albedo', icon: '🎭' },
    { id: 'lobstr', name: 'Lobstr Wallet', icon: '🦁' },
  ];

  useEffect(() => {
    // Detect network from current connection
    if (currentNetwork && currentNetwork !== 'unknown') {
      const network = Object.entries(NETWORKS).find(
        ([_, info]) => info.name.toLowerCase() === currentNetwork.toLowerCase()
      )?.[0] as NetworkType;
      setDetectedNetwork(network || 'unknown');
    }

    // Check for low balance (< 2 XLM)
    setLowBalance(currentBalance < 2);

    // Show warning if not on testnet and balance is low
    if (currentBalance < 2 && detectedNetwork !== 'testnet') {
      setShowNetworkWarning(true);
    }
  }, [currentNetwork, currentBalance, detectedNetwork]);

  const handleConnect = async (walletId: string) => {
    setSelectedWallet(walletId);
    setIsConnecting(true);

    try {
      await onConnect(walletId);
    } catch (error) {
      console.error('Connection failed:', error);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleFriendbot = async () => {
    if (!currentWallet) {
      setFundingStatus('error');
      setFundingMessage('No wallet connected');
      return;
    }

    if (detectedNetwork !== 'testnet' && detectedNetwork !== 'futurenet') {
      setFundingStatus('error');
      setFundingMessage('Friendbot only available on Testnet and Futurenet');
      return;
    }

    setIsFundingAccount(true);
    setFundingStatus('loading');
    setFundingMessage('Requesting XLM from Friendbot...');

    try {
      const networkInfo = NETWORKS[detectedNetwork];
      if (!networkInfo.friendbotUrl) {
        throw new Error('Friendbot not available for this network');
      }

      const response = await fetch(`${networkInfo.friendbotUrl}?addr=${currentWallet}`);

      if (!response.ok) {
        throw new Error(`Friendbot error: ${response.statusText}`);
      }

      const data = await response.json();

      // Verify successful funding
      if (data.hash) {
        setFundingStatus('success');
        setFundingMessage('Successfully funded with 10,000 XLM! Transaction: ' + data.hash.substring(0, 16) + '...');

        // Auto-close after 3 seconds
        setTimeout(() => {
          setFundingStatus('idle');
          setIsFundingAccount(false);
        }, 3000);
      } else {
        throw new Error('Funding failed - no transaction hash returned');
      }
    } catch (error) {
      setFundingStatus('error');
      setFundingMessage(
        error instanceof Error ? error.message : 'Failed to fund account with Friendbot'
      );
      setIsFundingAccount(false);
    }
  };

  const networkInfo = NETWORKS[detectedNetwork as NetworkType];
  const isCorrectNetwork = detectedNetwork === 'testnet' || detectedNetwork === 'futurenet';

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
      <div className="relative w-full max-w-md rounded-lg bg-white p-6 shadow-lg">
        {/* Close button */}
        <button
          onClick={onClose}
          className="absolute right-4 top-4 text-gray-400 hover:text-gray-600"
        >
          <X className="h-6 w-6" />
        </button>

        <h2 className="mb-2 text-2xl font-bold text-gray-900">Connect Wallet</h2>
        <p className="mb-6 text-sm text-gray-600">
          Choose your Stellar wallet to get started with InheritX
        </p>

        {/* Network detection warning */}
        {!isCorrectNetwork && (
          <div className="mb-4 flex items-start gap-2 rounded-lg bg-yellow-50 p-3">
            <AlertCircle className="mt-0.5 h-5 w-5 text-yellow-600 flex-shrink-0" />
            <div className="text-sm text-yellow-800">
              <p className="font-medium">Wrong Network Detected</p>
              <p className="mt-0.5">
                You're currently on <strong>{networkInfo.name}</strong>. Switch to Testnet or Futurenet
                for testing.
              </p>
            </div>
          </div>
        )}

        {/* Current wallet info */}
        {currentWallet && (
          <div className="mb-4 flex items-center justify-between rounded-lg bg-blue-50 p-3">
            <div>
              <p className="text-xs text-blue-600">CURRENT WALLET</p>
              <p className="font-mono text-sm text-gray-900">
                {currentWallet.substring(0, 8)}...{currentWallet.substring(-8)}
              </p>
              <p className="text-xs text-gray-600">
                Network: <strong>{networkInfo.name}</strong>
              </p>
            </div>
            <div className="text-right">
              <p className="text-xs text-gray-600">Balance</p>
              <p className={clsx('text-lg font-bold', lowBalance ? 'text-red-600' : 'text-green-600')}>
                {currentBalance.toFixed(2)}
              </p>
              <p className="text-xs text-gray-600">XLM</p>
            </div>
          </div>
        )}

        {/* Low balance warning */}
        {lowBalance && (
          <div className="mb-4 flex items-start gap-2 rounded-lg bg-red-50 p-3">
            <AlertCircle className="mt-0.5 h-5 w-5 text-red-600 flex-shrink-0" />
            <div className="text-sm text-red-800">
              <p className="font-medium">Low Balance</p>
              <p className="mt-0.5">
                You have less than 2 XLM. Request testnet XLM using Friendbot below.
              </p>
            </div>
          </div>
        )}

        {/* Wallet selection */}
        {!currentWallet && (
          <div className="mb-6 space-y-2">
            {walletOptions.map(wallet => (
              <button
                key={wallet.id}
                onClick={() => handleConnect(wallet.id)}
                disabled={isConnecting && selectedWallet !== wallet.id}
                className={clsx(
                  'w-full rounded-lg border-2 p-3 text-left transition-all',
                  selectedWallet === wallet.id && isConnecting
                    ? 'border-blue-500 bg-blue-50'
                    : 'border-gray-200 bg-white hover:border-gray-300'
                )}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-2xl">{wallet.icon}</span>
                    <div>
                      <p className="font-medium text-gray-900">{wallet.name}</p>
                      <p className="text-xs text-gray-600">Secure wallet connection</p>
                    </div>
                  </div>
                  {selectedWallet === wallet.id && isConnecting && (
                    <Loader className="h-5 w-5 animate-spin text-blue-600" />
                  )}
                </div>
              </button>
            ))}
          </div>
        )}

        {/* Friendbot section */}
        {currentWallet && lowBalance && isCorrectNetwork && (
          <div className="mb-6 space-y-3">
            <div className="rounded-lg bg-green-50 p-4">
              <div className="flex items-center gap-2 mb-2">
                <Zap className="h-5 w-5 text-green-600" />
                <h3 className="font-semibold text-green-900">Request Testnet XLM</h3>
              </div>
              <p className="text-sm text-green-800 mb-3">
                Use Friendbot to fund your testnet account with 10,000 XLM instantly.
              </p>
              <button
                onClick={handleFriendbot}
                disabled={isFundingAccount}
                className="w-full rounded-lg bg-green-600 px-4 py-2 font-medium text-white transition-colors hover:bg-green-700 disabled:bg-gray-400"
              >
                {isFundingAccount ? 'Requesting...' : 'Request Testnet XLM (Friendbot)'}
              </button>
            </div>

            {/* Funding status */}
            {fundingStatus !== 'idle' && (
              <div
                className={clsx(
                  'flex items-start gap-2 rounded-lg p-3',
                  fundingStatus === 'loading'
                    ? 'bg-blue-50 text-blue-800'
                    : fundingStatus === 'success'
                      ? 'bg-green-50 text-green-800'
                      : 'bg-red-50 text-red-800'
                )}
              >
                {fundingStatus === 'loading' && (
                  <Loader className="mt-0.5 h-5 w-5 animate-spin flex-shrink-0" />
                )}
                {fundingStatus === 'success' && (
                  <CheckCircle className="mt-0.5 h-5 w-5 flex-shrink-0" />
                )}
                {fundingStatus === 'error' && (
                  <AlertCircle className="mt-0.5 h-5 w-5 flex-shrink-0" />
                )}
                <p className="text-sm">{fundingMessage}</p>
              </div>
            )}
          </div>
        )}

        {/* Network info */}
        <div className="rounded-lg border border-gray-200 bg-gray-50 p-3 text-xs text-gray-600">
          <p className="font-medium text-gray-700 mb-2">Network Information</p>
          <div className="space-y-1 font-mono">
            <p>
              <span className="text-gray-500">Passphrase:</span> {networkInfo.passphrase}
            </p>
            <p>
              <span className="text-gray-500">RPC:</span> {networkInfo.rpcUrl}
            </p>
            {networkInfo.friendbotUrl && (
              <p>
                <span className="text-gray-500">Friendbot:</span> {networkInfo.friendbotUrl}
              </p>
            )}
          </div>
        </div>

        {/* Footer */}
        <p className="mt-4 text-center text-xs text-gray-500">
          By connecting, you agree to our Terms of Service
        </p>
      </div>
    </div>
  );
}

export default WalletModal;
