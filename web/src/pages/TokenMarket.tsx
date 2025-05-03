import React from 'react';
import { Box, Typography } from '@mui/material';
import Layout from '../components/Layout';
import TokenList from '../components/TokenList';

const TokenMarket: React.FC = () => {
  return (
    <Layout>
      <Box sx={{ p: 4 }}>
        <Typography variant="h4" gutterBottom>
          Token 市场
        </Typography>
        {/* 后续可添加 Token 市场相关表格或组件 */}
        <TokenList />
      </Box>
    </Layout>
  );
};

export default TokenMarket;