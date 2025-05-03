import React from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Typography,
  Box,
  Button,
} from '@mui/material';

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

interface TokenDetailDialogProps {
  open: boolean;
  onClose: () => void;
  tokenData?: TokenData;
}

const TokenDetailDialog: React.FC<TokenDetailDialogProps> = ({ open, onClose, tokenData }) => {
  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>Token 批次详情</DialogTitle>
      <DialogContent>
        {tokenData && Object.entries(tokenData).map(([key, value]) => (
          <Box key={key} sx={{ mb: 2 }}>
            <Typography variant="subtitle1">{key}:</Typography>
            <Typography>
              {typeof value === 'boolean' ? (value ? '是' : '否') :
                typeof value === 'bigint' ? value.toString() :
                value instanceof Date ? value.toLocaleDateString() :
                value}
            </Typography>
          </Box>
        ))}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>关闭</Button>
      </DialogActions>
    </Dialog>
  );
};

export default TokenDetailDialog;
