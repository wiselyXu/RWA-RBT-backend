import React, { useState } from 'react';
import { Box, TextField, Button, Typography } from '@mui/material';

// 定义 Token 数据类型
type TokenData = {
  tokenBatchNumber: string;
  stablecoin: string;
  availableShares: number;
};

// 定义 TokenPurchaseForm 组件的 Props 类型
type TokenPurchaseFormProps = {
  token: TokenData;
  onPurchase: (shares: number) => void;
  onClose: () => void;
};

const TokenPurchaseForm: React.FC<TokenPurchaseFormProps> = ({ token, onPurchase, onClose }) => {
  const [purchaseShares, setPurchaseShares] = useState(0);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    let value = e.target.value;
    // 去除前导 0
    value = value.replace(/^0+/, '');
    // 如果去除前导 0 后为空，设置为 0
    if (value === '') {
      value = '0';
    }
    const numValue = Math.min(Math.max(Number(value), 1), token.availableShares);
    setPurchaseShares(numValue);
  };

  const handlePurchase = () => {
    if (purchaseShares > 0 && purchaseShares <= token.availableShares) {
      onPurchase(purchaseShares);
      onClose();
    }
  };

  return (
    <Box sx={{ p: 2 }}>
      {/* 展示部分 */}
      <Box sx={{ mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          购买信息
        </Typography>
        <Typography>Token 批次号: {token.tokenBatchNumber}</Typography>
        <Typography>可售份额: {token.availableShares}</Typography>
        <Typography>稳定币: {token.stablecoin}</Typography>
      </Box>

      {/* 填写部分 */}
      <Box sx={{ mb: 3 }}>
        <TextField
          label="购买份额"
          type="number"
         // placeholder='0'
          value={purchaseShares}
          onChange={handleInputChange}
          fullWidth
          inputProps={{ min: 1, max: token.availableShares }}
        />
      </Box>

      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
        <Button variant="contained" onClick={handlePurchase} disabled={purchaseShares <= 0 || purchaseShares > token.availableShares}>
          确认购买
        </Button>
        <Button sx={{ ml: 1 }} onClick={onClose}>
          取消
        </Button>
      </Box>
    </Box>
  );
};

export default TokenPurchaseForm;
