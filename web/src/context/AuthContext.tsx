import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import AuthService from '../services/authService';
import WalletService from '../services/walletService';
import ApiService from '../services/apiService';

// Define user roles
export enum UserRole {
  Investor = 'Investor',
  EnterpriseAdmin = 'EnterpriseAdmin',
  PlatformAdmin = 'PlatformAdmin'
}

// 企业信息接口
interface EnterpriseInfo {
  isEnterpriseBound: boolean;
  enterpriseName?: string;
  enterpriseAddress?: string;
  enterpriseId?: string;
}

interface UserInfo {
  walletAddress: string;
  role: UserRole;
  isEnterpriseBound: boolean;
  enterpriseId?: string;
  enterpriseName?: string;
  enterpriseAddress?: string;
}

interface EnterpriseBindRequest {
  enterpriseAddress: string;
}

interface AuthContextType {
  isAuthenticated: boolean;
  userInfo: UserInfo | null;
  isLoading: boolean;
  isBindingEnterprise: boolean;
  login: (walletType: string, address: string) => Promise<boolean>;
  logout: () => void;
  bindEnterprise: (enterpriseAddress: string) => Promise<boolean>;
  fetchEnterpriseInfo: () => Promise<EnterpriseInfo | null>;
}

// Create context with default values
const AuthContext = createContext<AuthContextType>({
  isAuthenticated: false,
  userInfo: null,
  isLoading: false,
  isBindingEnterprise: false,
  login: async () => false,
  logout: () => {},
  bindEnterprise: async () => false,
  fetchEnterpriseInfo: async () => null,
});

// Custom hook to use the auth context
export const useAuth = () => useContext(AuthContext);

interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [isAuthenticated, setIsAuthenticated] = useState<boolean>(false);
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isBindingEnterprise, setIsBindingEnterprise] = useState<boolean>(false);
  
  const authService = AuthService.getInstance();
  const walletService = WalletService.getInstance();
  const apiService = ApiService.getInstance();

  // 获取用户绑定的企业信息
  const fetchEnterpriseInfo = async (): Promise<EnterpriseInfo | null> => {
    try {
      if (!authService.isAuthenticated()) {
        return null;
      }

      const enterpriseInfo = await apiService.get<EnterpriseInfo>('/user/enterprise-info');
      console.log('获取企业信息成功:', enterpriseInfo);
      
      if (userInfo && enterpriseInfo.isEnterpriseBound) {
        // 更新用户信息中的企业数据
        setUserInfo({
          ...userInfo,
          isEnterpriseBound: true,
          role: UserRole.EnterpriseAdmin,
          enterpriseId: enterpriseInfo.enterpriseId,
          enterpriseName: enterpriseInfo.enterpriseName,
          enterpriseAddress: enterpriseInfo.enterpriseAddress
        });
      }
      
      return enterpriseInfo;
    } catch (error) {
      console.error('获取企业信息失败:', error);
      return null;
    }
  };

  // Check if a user has an enterprise bound
  const checkEnterpriseBinding = async (walletAddress: string): Promise<boolean> => {
    try {
      // Call the API to get enterprise info
      const enterpriseInfo = await fetchEnterpriseInfo();
      return enterpriseInfo?.isEnterpriseBound || false;
    } catch (error) {
      console.error('Error checking enterprise binding:', error);
      return false;
    }
  };

  // Check authentication status on initial load
  useEffect(() => {
    const checkAuth = async () => {
      // Check if user has a valid token
      const isUserAuthenticated = authService.isAuthenticated();
      setIsAuthenticated(isUserAuthenticated);
      
      // If authenticated, also try to get current wallet and check enterprise binding
      if (isUserAuthenticated) {
        const wallet = walletService.getCurrentWallet();
        if (wallet?.address) {
          // 获取企业绑定信息
          const enterpriseInfo = await fetchEnterpriseInfo();
          const isEnterpriseBound = enterpriseInfo?.isEnterpriseBound || false;
          
          setUserInfo({
            walletAddress: wallet.address,
            role: isEnterpriseBound ? UserRole.EnterpriseAdmin : UserRole.Investor,
            isEnterpriseBound,
            enterpriseId: enterpriseInfo?.enterpriseId,
            enterpriseName: enterpriseInfo?.enterpriseName,
            enterpriseAddress: enterpriseInfo?.enterpriseAddress
          });
        }
      } else {
        setUserInfo(null);
      }
    };
    
    checkAuth();
  }, []);

  // Handle wallet authentication
  const login = async (walletType: string, address: string): Promise<boolean> => {
    try {
      setIsLoading(true);
      
      // 1. Request challenge
      const challenge = await authService.requestChallenge(address);
      
      // 2. Authenticate with challenge
      await authService.login(walletType, challenge.requestId, challenge.nonce);
      
      // 3. Check enterprise binding status
      const enterpriseInfo = await fetchEnterpriseInfo();
      const isEnterpriseBound = enterpriseInfo?.isEnterpriseBound || false;
      
      // 4. Update authentication state
      setIsAuthenticated(true);
      setUserInfo({
        walletAddress: address,
        role: isEnterpriseBound ? UserRole.EnterpriseAdmin : UserRole.Investor,
        isEnterpriseBound,
        enterpriseId: enterpriseInfo?.enterpriseId,
        enterpriseName: enterpriseInfo?.enterpriseName,
        enterpriseAddress: enterpriseInfo?.enterpriseAddress
      });
      
      return true;
    } catch (error) {
      console.error('Authentication failed:', error);
      return false;
    } finally {
      setIsLoading(false);
    }
  };

  // Handle logout
  const logout = () => {
    authService.logout();
    setIsAuthenticated(false);
    setUserInfo(null);
  };

  // Handle binding enterprise
  const bindEnterprise = async (enterpriseAddress: string): Promise<boolean> => {
    if (!userInfo?.walletAddress) {
      return false;
    }
    
    try {
      setIsBindingEnterprise(true);
      
      // Call the bind-enterprise endpoint
      await apiService.post('/user/bind-enterprise', {
        enterpriseAddress
      });
      
      // Get updated enterprise info
      const enterpriseInfo = await fetchEnterpriseInfo();
      
      if (enterpriseInfo?.isEnterpriseBound) {
        // Update user info with enterprise details
        setUserInfo({
          ...userInfo,
          role: UserRole.EnterpriseAdmin,
          isEnterpriseBound: true,
          enterpriseId: enterpriseInfo.enterpriseId,
          enterpriseName: enterpriseInfo.enterpriseName,
          enterpriseAddress: enterpriseInfo.enterpriseAddress
        });
        
        return true;
      }
      
      return false;
    } catch (error) {
      console.error('Failed to bind enterprise:', error);
      return false;
    } finally {
      setIsBindingEnterprise(false);
    }
  };

  const value = {
    isAuthenticated,
    userInfo,
    isLoading,
    isBindingEnterprise,
    login,
    logout,
    bindEnterprise,
    fetchEnterpriseInfo
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export default AuthContext; 