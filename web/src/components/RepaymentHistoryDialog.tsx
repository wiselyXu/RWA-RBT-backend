import React from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  Button,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
} from '@mui/material';

// 定义偿还历史数据类型
type RepaymentHistoryItem = {
  repaymentDate: string;
  repaymentAmount: number;
};

// 定义 RepaymentHistoryDialog 组件的 Props 类型
type RepaymentHistoryDialogProps = {
  open: boolean;
  onClose: () => void;
  tokenBatchNumber: string;
  history: RepaymentHistoryItem[];
};

const RepaymentHistoryDialog: React.FC<RepaymentHistoryDialogProps> = ({
  open,
  onClose,
  tokenBatchNumber,
  history,
}) => {
  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>批次 {tokenBatchNumber} 偿还历史</DialogTitle>
      <DialogContent>
        <TableContainer component={Paper}>
          <Table aria-label="repayment history">
            <TableHead>
              <TableRow>
                <TableCell>偿还日期</TableCell>
                <TableCell>偿还金额</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {history.map((item, index) => (
                <TableRow key={index}>
                  <TableCell>{item.repaymentDate}</TableCell>
                  <TableCell>{item.repaymentAmount}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>关闭</Button>
      </DialogActions>
    </Dialog>
  );
};

export default RepaymentHistoryDialog;
