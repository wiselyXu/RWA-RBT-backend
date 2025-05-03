import WalletService from './walletService';

interface ChallengeResponse {
  nonce: string;
  requestId: string;
}

interface LoginResponse {
  token: string;
  walletAddress: string;
}

// API base URL
const API_BASE_URL = '/rwa'; // This matches the API prefix configured on the backend

class AuthService {
  private static instance: AuthService;
  private token: string | null = null;
  private readonly TOKEN_KEY = 'auth_token';

  private constructor() {
    // Try to load the token from localStorage on initialization
    this.token = localStorage.getItem(this.TOKEN_KEY);
    if (this.token) {
      console.log('Auth token loaded from storage');
    }
  }

  public static getInstance(): AuthService {
    if (!AuthService.instance) {
      AuthService.instance = new AuthService();
    }
    return AuthService.instance;
  }

  // Request a challenge from the server
  public async requestChallenge(walletAddress: string): Promise<ChallengeResponse> {
    try {
      console.log(`Requesting challenge for address: ${walletAddress}`);
      
      // Request a challenge from the server
      const challengeUrl = `${API_BASE_URL}/user/challenge`;
      const response = await fetch(challengeUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          walletAddress,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        console.error('Challenge request failed:', errorData);
        throw new Error(errorData.message || 'Challenge request failed');
      }

      const data = await response.json();
      const { nonce, requestId } = data.data as ChallengeResponse;
      console.log(`Challenge received. RequestId: ${requestId}, Nonce: ${nonce}`);
      
      return { nonce, requestId };
    } catch (error) {
      console.error('Failed to get challenge:', error);
      throw error;
    }
  }

  // Sign the challenge with the wallet and login
  public async login(walletType: string, requestId: string, nonce: string): Promise<string> {
    try {
      console.log(`Login attempt with requestId: ${requestId}, nonce: ${nonce}`);
      
      // Get wallet service instance
      const walletService = WalletService.getInstance();
      
      // Get current wallet info
      const walletInfo = walletService.getCurrentWallet();
      if (!walletInfo) {
        throw new Error('No wallet connected');
      }

      console.log(`Signing message with wallet type: ${walletType}, address: ${walletInfo.address}`);
      
      // Sign the message (nonce) with the wallet
      let signature: string;
      
      switch (walletType) {
        case 'metamask':
          if (!window.ethereum) {
            throw new Error('MetaMask is not installed');
          }
          // Use personal_sign to sign the nonce
          signature = await window.ethereum.request({
            method: 'personal_sign',
            params: [nonce, walletInfo.address],
          });
          break;

        case 'okx':
          if (!window.okxwallet) {
            throw new Error('OKX Wallet is not installed');
          }
          signature = await window.okxwallet.request({
            method: 'personal_sign',
            params: [nonce, walletInfo.address],
          });
          break;

        case 'bitget':
          if (!window.bitkeep) {
            throw new Error('Bitget Wallet is not installed');
          }
          signature = await window.bitkeep.request({
            method: 'personal_sign',
            params: [nonce, walletInfo.address],
          });
          break;

        default:
          throw new Error('Unsupported wallet type for signing');
      }

      console.log(`Message signed. Signature length: ${signature.length}`);
      console.log(`Submitting login request to ${API_BASE_URL}/user/login`);

      // Send the signature to the login endpoint
      const loginUrl = `${API_BASE_URL}/user/login`;
      const response = await fetch(loginUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          requestId,
          signature,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        console.error('Login request failed:', errorData);
        throw new Error(errorData.message || 'Login failed');
      }

      const data = await response.json();
      console.log('Login successful. Token received.');
      const { token } = data.data as LoginResponse;

      // Store the token
      this.token = token;
      localStorage.setItem(this.TOKEN_KEY, token);

      return token;
    } catch (error) {
      console.error('Login failed:', error);
      throw error;
    }
  }

  // Helper method to get the stored token
  public getToken(): string | null {
    return this.token;
  }

  // Check if user is authenticated
  public isAuthenticated(): boolean {
    return !!this.token;
  }

  // Logout method
  public logout(): void {
    console.log('Logging out user, clearing token');
    this.token = null;
    localStorage.removeItem(this.TOKEN_KEY);
    // Also disconnect the wallet
    WalletService.getInstance().disconnectWallet();
  }

  // Get auth header for API requests
  public getAuthHeader(): { Authorization: string } | {} {
    if (this.token) {
      return { Authorization: `Bearer ${this.token}` };
    }
    return {};
  }
}

export default AuthService; 