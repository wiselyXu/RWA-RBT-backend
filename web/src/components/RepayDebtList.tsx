import React, { useState, useEffect } from 'react';
import {
  Box,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Typography,
  Pagination,
  Link,
  Tooltip,
  IconButton,
} from '@mui/material';
import HistoryIcon from '@mui/icons-material/History';
import Layout from './Layout';
import DebtDetailDialog from './DebtDetailDialog';
import RepaymentHistoryDialog from './RepaymentHistoryDialog';

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

// 定义偿还历史数据类型
type RepaymentHistoryItem = {
  repaymentDate: string;
  repaymentAmount: number;
};

// 定义 RepayDebtList 组件的 Props 类型
type RepayDebtListProps = {};

const RepayDebtList: React.FC<RepayDebtListProps> = () => {
  const [debtData, setDebtData] = useState<DebtData[]>([]);
  const [openRepayDialog, setOpenRepayDialog] = useState(false);
  const [selectedDebt, setSelectedDebt] = useState<DebtData | null>(null);
  const [repayAmount, setRepayAmount] = useState(0);
  const [page, setPage] = useState(1);
  const rowsPerPage = 5;
  const [openDetailDialog, setOpenDetailDialog] = useState(false);
  const [openHistoryDialog, setOpenHistoryDialog] = useState(false);
  const [selectedTokenBatchNumber, setSelectedTokenBatchNumber] = useState('');
  const [repaymentHistory, setRepaymentHistory] = useState<RepaymentHistoryItem[]>([]);

  // 模拟获取债务数据
  useEffect(() => {
    const mockData: DebtData[] = Array.from({ length: 20 }, (_, index) => ({
      tokenBatchNumber: `Batch-${String(index + 1).padStart(3, '0')}`,
      stablecoin: index % 2 === 0 ? 'USDT' : 'MNT',
      issuedShares: (index + 1) * 1000,
      accumulatedShares: (index + 1) * 1100,
      interestShares: (index + 1) * 100,
      remainingShares: Math.floor(Math.random() * (index + 1) * 500),
      repaidShares: (index + 1) * 1000 - Math.floor(Math.random() * (index + 1) * 500),
    }));
    setDebtData(mockData);
  }, []);

  const handleChangePage = (event: React.ChangeEvent<unknown>, newPage: number) => {
    setPage(newPage);
  };

  const handleRepayClick = (debt: DebtData) => {
    setSelectedDebt(debt);
    setOpenRepayDialog(true);
    setRepayAmount(0);
  };

  const handleCloseRepayDialog = () => {
    setOpenRepayDialog(false);
    setSelectedDebt(null);
  };

  const handleConfirmRepay = () => {
    if (repayAmount > 0 && repayAmount <= (selectedDebt?.remainingShares || 0)) {
      setDebtData((prevData) =>
        prevData.map((item) =>
          item.tokenBatchNumber === selectedDebt?.tokenBatchNumber
            ? {
                ...item,
                remainingShares: item.remainingShares - repayAmount,
                repaidShares: item.repaidShares + repayAmount,
              }
            : item
        )
      );
      handleCloseRepayDialog();
    }
  };

  const handleOpenDetailDialog = (debt: DebtData) => {
    setSelectedDebt(debt);
    setOpenDetailDialog(true);
  };

  const handleCloseDetailDialog = () => {
    setOpenDetailDialog(false);
  };

  const handleOpenHistoryDialog = (tokenBatchNumber: string) => {
    setSelectedTokenBatchNumber(tokenBatchNumber);
    // 模拟获取偿还历史数据
    const mockHistory: RepaymentHistoryItem[] = Array.from({ length: 3 }, (_, index) => ({
      repaymentDate: new Date(Date.now() - index * 86400000).toISOString().split('T')[0],
      repaymentAmount: Math.floor(Math.random() * 1000) + 100,
    }));
    setRepaymentHistory(mockHistory);
    setOpenHistoryDialog(true);
  };

  const handleCloseHistoryDialog = () => {
    setOpenHistoryDialog(false);
  };

  const indexOfLastRow = page * rowsPerPage;
  const indexOfFirstRow = indexOfLastRow - rowsPerPage;
  const currentRows = debtData.slice(indexOfFirstRow, indexOfLastRow);

  return (
    <Layout>
      <Box>
        <TableContainer component={Paper}>
          <Table aria-label="repay debt list">
            <TableHead>
              <TableRow>
                <TableCell>token批次编号</TableCell>
                <TableCell>稳定币</TableCell>
                <TableCell>发行份额</TableCell>
                <TableCell>累计份额</TableCell>
                <TableCell>利息份额</TableCell>
                <TableCell>剩余份额</TableCell>
                <TableCell>已还份额</TableCell>
                <TableCell>操作</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {currentRows.map((row) => (
                <TableRow key={row.tokenBatchNumber}>
                  <TableCell>
                    <Link
                      href="#"
                      onClick={(e) => {
                        e.preventDefault();
                        handleOpenDetailDialog(row);
                      }}
                    >
                      {row.tokenBatchNumber}
                    </Link>
                  </TableCell>
                  <TableCell>{row.stablecoin}</TableCell>
                  <TableCell>{row.issuedShares}</TableCell>
                  <TableCell>{row.accumulatedShares}</TableCell>
                  <TableCell>{row.interestShares}</TableCell>
                  <TableCell>{row.remainingShares}</TableCell>
                  <TableCell>{row.repaidShares}</TableCell>
                  <TableCell>
                    <Button
                      variant="contained"
                      onClick={() => handleRepayClick(row)}
                      disabled={row.remainingShares <= 0}
                    >
                      偿还
                    </Button>
                    <Tooltip title="查看偿还历史">
                      <IconButton onClick={() => handleOpenHistoryDialog(row.tokenBatchNumber)}>
                        <HistoryIcon />
                      </IconButton>
                    </Tooltip>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
        <Box sx={{ display: 'flex', justifyContent: 'center', mt: 2 }}>
          <Pagination
            count={Math.ceil(debtData.length / rowsPerPage)}
            page={page}
            onChange={handleChangePage}
            variant="outlined"
            shape="rounded"
          />
        </Box>
        <Dialog open={openRepayDialog} onClose={handleCloseRepayDialog}>
          <DialogTitle>偿还债务</DialogTitle>
          <DialogContent>
            {selectedDebt && (
              <>
                <Typography>Token 批次号: {selectedDebt.tokenBatchNumber}</Typography>
                <Typography>稳定币: {selectedDebt.stablecoin}</Typography>
                <Typography>剩余份额: {selectedDebt.remainingShares}</Typography>
                <TextField
                  label="偿还份额"
                  type="number"
                  value={repayAmount}
                  onChange={(e) => setRepayAmount(Number(e.target.value))}
                  fullWidth
                  inputProps={{ min: 1, max: selectedDebt.remainingShares }}
                  margin="normal"
                />
              </>
            )}
          </DialogContent>
          <DialogActions>
            <Button onClick={handleCloseRepayDialog}>取消</Button>
            <Button onClick={handleConfirmRepay} disabled={repayAmount <= 0 || repayAmount > (selectedDebt?.remainingShares || 0)}>
              确认偿还
            </Button>
          </DialogActions>
        </Dialog>
        <DebtDetailDialog
          open={openDetailDialog}
          onClose={handleCloseDetailDialog}
          debtData={selectedDebt}
        />
        <RepaymentHistoryDialog
          open={openHistoryDialog}
          onClose={handleCloseHistoryDialog}
          tokenBatchNumber={selectedTokenBatchNumber}
          history={repaymentHistory}
        />
      </Box>
    </Layout>
  );
};

export default RepayDebtList;
