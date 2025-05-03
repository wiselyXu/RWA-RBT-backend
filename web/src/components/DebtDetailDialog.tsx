import React from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  Button,
  Typography,
} from '@mui/material';

// 定义债务数据类型
type DebtData = {
  tokenBatchNumber: string;
  stablecoin: string;
  issuedShares: number;
  accumulatedShares: number;
  interestShares: number;
  remainingShares: number;
  repaidShares: number;
};

// 定义 DebtDetailDialog 组件的 Props 类型
type DebtDetailDialogProps = {
  open: boolean;
  onClose: () => void;
  debtData: DebtData | null;
};

const DebtDetailDialog: React.FC<DebtDetailDialogProps> = ({ open, onClose, debtData }) => {
  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>债务详情</DialogTitle>
      <DialogContent>
        {debtData && (
          <>
            <Typography>Token 批次号: {debtData.tokenBatchNumber}</Typography>
            <Typography>稳定币: {debtData.stablecoin}</Typography>
            <Typography>发行份额: {debtData.issuedShares}</Typography>
            <Typography>累计份额: {debtData.accumulatedShares}</Typography>
            <Typography>利息份额: {debtData.interestShares}</Typography>
            <Typography>剩余份额: {debtData.remainingShares}</Typography>
            <Typography>已还份额: {debtData.repaidShares}</Typography>
          </>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>关闭</Button>
      </DialogActions>
    </Dialog>
  );
};

export default DebtDetailDialog;
