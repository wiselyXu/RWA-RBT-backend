import React, { useEffect, useState } from 'react';
import { Box, Typography } from '@mui/material';
import TokenTable from '../components/TokenTable';
import Layout from '../components/Layout';

// 模拟 token 数据，实际开发中应从 API 获取

type TokenData = {
    tokenBatchNumber: string;
    creditorAccount: string;
    debtor: string;
    stablecoin: string;
    billQuantity: number;
    issuedAmount: bigint;
    isDebtorSigned: boolean;
    createTime: Date;
    createWallet: string;
    modifyTime: Date;
    modifyWallet: string;
  };

const generateMockTokenData = (): TokenData[] => {
    const data: TokenData[] = [];
    for (let i = 1; i <= 10; i++) {
      data.push({
        tokenBatchNumber: `Batch-${i}`,
        creditorAccount: `Creditor-${i}`,
        debtor: `Debtor-${i}`,
        stablecoin: `Stablecoin-${i}`,
        billQuantity: i * 10,
        issuedAmount: BigInt(i * 1000),
        isDebtorSigned: i % 2 === 0,
        createTime: new Date(),
        createWallet: `CreateWallet-${i}`,
        modifyTime: new Date(),
        modifyWallet: `ModifyWallet-${i}`,
      });
    }
    return data;
  };

const mockTokenData =  generateMockTokenData();

const MyIssuedTokens: React.FC = () => {
  const [tokenData, setTokenData] = useState<{
    tokenBatchNumber: string;
    creditorAccount: string;
    debtor: string;
    stablecoin: string;
    billQuantity: number;
    issuedAmount: bigint;
    isDebtorSigned: boolean;
    createTime: Date;
    createWallet: string;
    modifyTime: Date;
    modifyWallet: string;
  }[]>([]);

  useEffect(() => {
    // 模拟从 API 获取数据
    setTokenData(mockTokenData);
  }, []);

  return (
    <Layout>
        <Box sx={{ p: 4 }}>
         <Typography variant="h4" gutterBottom>
        Token 管理
        </Typography>
        <TokenTable data={tokenData} />
        </Box>
    </Layout>
  );
};

export default MyIssuedTokens;
