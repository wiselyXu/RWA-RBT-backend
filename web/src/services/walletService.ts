import { InjectedConnector } from '@web3-react/injected-connector';
import { WalletConnectConnector } from '@web3-react/walletconnect-connector';

// 支持的链ID
const supportedChainIds = [1, 56, 137]; // Ethereum, BSC, Polygon

// MetaMask连接器
export const injected = new InjectedConnector({
  supportedChainIds,
});

// WalletConnect连接器
export const walletconnect = new WalletConnectConnector({
  rpc: {
    1: 'https://mainnet.infura.io/v3/YOUR_INFURA_KEY',
    56: 'https://bsc-dataseed.binance.org',
    137: 'https://polygon-rpc.com',
  },
  bridge: 'https://bridge.walletconnect.org',
  qrcode: true,
  supportedChainIds,
});

export type WalletType = 'metamask' | 'walletconnect' | 'okx' | 'bitget';

export interface WalletInfo {
  address: string;
  type: WalletType;
}

class WalletService {
  private static instance: WalletService;
  private currentWallet: WalletInfo | null = null;
  private readonly STORAGE_KEY = 'wallet_info';

  private constructor() {
    // 从 localStorage 恢复钱包状态
    const savedWallet = localStorage.getItem(this.STORAGE_KEY);
    if (savedWallet) {
      try {
        this.currentWallet = JSON.parse(savedWallet);
      } catch (error) {
        console.error('Failed to parse saved wallet:', error);
        localStorage.removeItem(this.STORAGE_KEY);
      }
    }
  }

  public static getInstance(): WalletService {
    if (!WalletService.instance) {
      WalletService.instance = new WalletService();
    }
    return WalletService.instance;
  }

  public async connectWallet(type: WalletType): Promise<WalletInfo> {
    try {
      let address: string;

      switch (type) {
        case 'metamask':
          if (!window.ethereum) {
            throw new Error('MetaMask is not installed');
          }
          const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
          address = accounts[0];
          break;

        case 'walletconnect':
          // Implement WalletConnect logic here
          throw new Error('WalletConnect not implemented yet');

        case 'okx':
          if (!window.okxwallet) {
            throw new Error('OKX Wallet is not installed');
          }
          const okxAccounts = await window.okxwallet.request({ method: 'eth_requestAccounts' });
          address = okxAccounts[0];
          break;

        case 'bitget':
          if (!window.bitkeep) {
            throw new Error('Bitget Wallet is not installed');
          }
          const bitgetAccounts = await window.bitkeep.request({ method: 'eth_requestAccounts' });
          address = bitgetAccounts[0];
          break;

        default:
          throw new Error('Unsupported wallet type');
      }

      this.currentWallet = { address, type };
      // 保存到 localStorage
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(this.currentWallet));
      return this.currentWallet;
    } catch (error) {
      console.error('Failed to connect wallet:', error);
      throw error;
    }
  }

  public disconnectWallet(): void {
    this.currentWallet = null;
    // 从 localStorage 中移除
    localStorage.removeItem(this.STORAGE_KEY);
  }

  public getCurrentWallet(): WalletInfo | null {
    return this.currentWallet;
  }

  public isConnected(): boolean {
    return this.currentWallet !== null;
  }
}

// Extend Window interface to include wallet providers
declare global {
  interface Window {
    ethereum?: any;
    okxwallet?: any;
    bitkeep?: any;
  }
}

export default WalletService; 