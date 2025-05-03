import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import BillManagement from './components/BillManagement';
import HomePage from './components/HomePage'; 
import MyIssuedTokens from './pages/MyIssuedTokens';
import TokenMarket from './pages/TokenMarket';
import RepayDebtList from './components/RepayDebtList';
import PendingSignatureList from './components/PendingSignatureList';
import MyTokenList from './components/MyTokenList';
import InvoiceManagement from './components/InvoiceManagement';
import { AuthProvider, useAuth } from './context/AuthContext';

const handleSign = (data: {
  tokenBatchNumber: string;
  creditorAccount: string;
  debtor: string;
  stablecoin: string;
  billQuantity: number;
  issuedAmount: bigint;
}) => {
  console.log('执行签名操作，数据为:', data);
  alert('调钱包操作， 钱包操作时给信息 =>' +JSON.stringify(data));
  // 可以在这里添加实际的签名逻辑
};

// Protected route component
const ProtectedRoute: React.FC<{ element: React.ReactElement }> = ({ element }) => {
  const { isAuthenticated, isLoading } = useAuth();
  
  // Show loading state while checking authentication
  if (isLoading) {
    return <div>Loading...</div>;
  }
  
  // Redirect to home if not authenticated
  return isAuthenticated ? element : <Navigate to="/" />;
};

// Main App component
const AppRoutes: React.FC = () => {
  return (
    <Routes>
      <Route path="/" element={<HomePage />} />
      
      {/* Protected routes that require authentication */}
      <Route path="/my-credits/my-bills" element={<ProtectedRoute element={<BillManagement />} />} />
      <Route path="/my-credits/invoices" element={<ProtectedRoute element={<InvoiceManagement />} />} />
      <Route path="/my-credits/my-issued-tokens" element={<ProtectedRoute element={<MyIssuedTokens />} />} />
      <Route path="token-market" element={<ProtectedRoute element={<TokenMarket />} />} />
      <Route path="/my-debts/repay-debt" element={<ProtectedRoute element={<RepayDebtList />} />} />
      <Route path="/my-debts/my-todolist" element={<ProtectedRoute element={<PendingSignatureList onSign={handleSign} />} />} />
      <Route path="/my-tokens" element={<ProtectedRoute element={<MyTokenList />} />} />
    </Routes>
  );
};

// Wrap the app with necessary providers
const App: React.FC = () => {
  return (
    <AuthProvider>
    <Router>
        <AppRoutes />
    </Router>
    </AuthProvider>
  );
};

export default App;
