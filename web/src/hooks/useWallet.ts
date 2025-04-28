import { useState, useEffect, useCallback } from 'react';
import { ethers } from 'ethers';
import { getChallenge, loginWithSignature } from '../services/api';

// Define the state structure for the hook
export interface WalletState {
  address: string | null;
  token: string | null;
  isLoading: boolean;
  error: string | null;
  connectWallet: () => Promise<void>;
  disconnectWallet: () => void;
}

export function useWallet(): WalletState {
  const [address, setAddress] = useState<string | null>(null);
  const [token, setToken] = useState<string | null>(localStorage.getItem('authToken')); // Initialize token from localStorage
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [provider, setProvider] = useState<ethers.BrowserProvider | null>(null);

  // Effect to initialize provider and check initial connection/token
  useEffect(() => {
    if (window.ethereum) {
      const browserProvider = new ethers.BrowserProvider(window.ethereum);
      setProvider(browserProvider);

      // Check if already connected on load
      browserProvider.listAccounts().then(accounts => {
          if (accounts.length > 0) {
              const userAddress = accounts[0].address;
              setAddress(userAddress);
              console.log('Wallet already connected:', userAddress);
              // If connected but no token, maybe prompt login or just show address
              if (!localStorage.getItem('authToken')) {
                  console.log('Connected but no token found.');
                  // Optionally trigger login automatically here if desired
              }
          }
      }).catch(err => {
          console.error('Error checking initial connection:', err);
          // Don't necessarily set an error here, maybe the user just hasn't connected yet
      });

      const handleAccountsChanged = (accounts: string[]) => {
        if (accounts.length === 0) {
          console.log('Wallet disconnected by user.');
          disconnectWallet();
        } else {
          const newAddress = accounts[0];
          console.log('Account changed:', newAddress);
          // If address changes, treat as disconnect/reconnect needed for new login
          disconnectWallet(); // Clear old state
          setAddress(newAddress); // Set new address immediately for UI
          // Optionally auto-trigger login: connectWallet();
        }
      };

      const handleChainChanged = (_chainId: string) => {
        console.log('Network changed, reloading...');
        window.location.reload();
      };

      window.ethereum.on('accountsChanged', handleAccountsChanged);
      window.ethereum.on('chainChanged', handleChainChanged);

      // Cleanup listeners on unmount
      return () => {
        if (window.ethereum.removeListener) {
          window.ethereum.removeListener('accountsChanged', handleAccountsChanged);
          window.ethereum.removeListener('chainChanged', handleChainChanged);
        }
      };
    } else {
      console.log('MetaMask not detected');
    }
  }, []); // Run only once on mount

  const connectWallet = useCallback(async () => {
    if (!provider) {
      setError('Wallet provider not found. Please install MetaMask.');
      console.error('Provider not available');
      alert('MetaMask is not installed or not detected. Please install MetaMask and refresh the page.'); // User feedback
      return;
    }

    // Ensure previous errors are cleared
    setError(null); 
    setIsLoading(true);

    try {
      // Request account access explicitly
      await provider.send("eth_requestAccounts", []); 
      const signer = await provider.getSigner();
      const userAddress = await signer.getAddress();
      setAddress(userAddress);
      console.log('Wallet connected:', userAddress);

      // --- Login Flow ---
      console.log('Requesting challenge...');
      const challengeRes = await getChallenge(userAddress);

      if (challengeRes.code !== 200 || !challengeRes.data) {
        throw new Error(challengeRes.msg || 'Failed to get challenge');
      }

      const { nonce, requestId } = challengeRes.data;
      console.log('Received nonce:', nonce);

      console.log('Requesting signature...');
      const signature = await signer.signMessage(nonce);
      console.log('Signature obtained');

      console.log('Sending signature for login...');
      const loginRes = await loginWithSignature(requestId, signature);

      if (loginRes.code !== 200 || !loginRes.data) {
        throw new Error(loginRes.msg || 'Login failed');
      }

      const receivedToken = loginRes.data.token;
      setToken(receivedToken);
      localStorage.setItem('authToken', receivedToken);
      console.log('Login successful, token received and stored.');

    } catch (err: any) {
      console.error('Connection/Login failed:', err);
      let errorMessage = 'An unknown error occurred.';
      if (err.message) {
        errorMessage = err.message;
      }
      // Specific user-friendly messages
      if (err.code === 4001) { // EIP-1193 user rejected request
          errorMessage = 'Connection/Signature request rejected by user.';
      }
      setError(errorMessage);
      // Clear potentially partial state on error
      setAddress(null);
      setToken(null);
      localStorage.removeItem('authToken');
    } finally {
      setIsLoading(false);
    }
  }, [provider]);

  const disconnectWallet = useCallback(() => {
    setAddress(null);
    setToken(null);
    localStorage.removeItem('authToken');
    console.log('Wallet disconnected (state cleared)');
    // No need to directly interact with provider for disconnection usually,
    // relies on user action in MetaMask or account change event.
  }, []);

  return { address, token, isLoading, error, connectWallet, disconnectWallet };
} 