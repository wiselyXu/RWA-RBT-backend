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
  Pagination,
  Button,
  Link,
  Tooltip,
  Dialog,
  DialogTitle,
} from '@mui/material';
import { Visibility } from '@mui/icons-material';
import TokenPurchaseForm from './TokenPurchaseForm';

// 定义 Token 数据类型
type TokenData = {
  tokenBatchNumber: string;
  creditor: string;
  debtor: string;
  stablecoin: string;
  issuedShares: number;
  totalAccumulatedShares: number;
  totalValidShares: number;
  soldShares: number;
  availableShares: number;
  totalRepaidShares: number;
};

// 生成模拟数据
const generateMockTokenData = (): TokenData[] => {
  const data: TokenData[] = [];
  for (let i = 1; i <= 20; i++) {
    const issuedShares = i * 100;
    const soldShares = Math.floor(issuedShares * Math.random());
    const totalRepaidShares = Math.floor(soldShares * Math.random());
    data.push({
      tokenBatchNumber: `Batch-${i}`,
      creditor: `0x${Math.random().toString(16).substr(2, 40)}`,
      debtor: `0x${Math.random().toString(16).substr(2, 40)}`,
      stablecoin: i%3===0?`USDT`:i%2===0?`HKDC`:`MNT`,
      issuedShares,
      totalAccumulatedShares: issuedShares,
      totalValidShares: issuedShares - soldShares,
      soldShares,
      availableShares: issuedShares - soldShares,
      totalRepaidShares,
    });
  }
  return data;
};

const TokenList: React.FC = () => {
  const [tokenData, setTokenData] = useState<TokenData[]>([]);
  const [page, setPage] = useState(1);
  const rowsPerPage = 10;
  const [openPurchaseDialog, setOpenPurchaseDialog] = useState(false);
  const [selectedToken, setSelectedToken] = useState<TokenData | null>(null);

  useEffect(() => {
    const mockData = generateMockTokenData();
    setTokenData(mockData);
  }, []);

  const handleChangePage = (event: React.ChangeEvent<unknown>, newPage: number) => {
    setPage(newPage);
  };

  const indexOfLastRow = page * rowsPerPage;
  const indexOfFirstRow = indexOfLastRow - rowsPerPage;
  const currentRows = tokenData.slice(indexOfFirstRow, indexOfLastRow);

  const handleBuy = (token: TokenData) => {
    setSelectedToken(token);
    setOpenPurchaseDialog(true);
  };

  const handleViewDetails = (token: TokenData) => {
    // 原有查看详情逻辑
  };

  const handleClosePurchaseDialog = () => {
    setOpenPurchaseDialog(false);
    setSelectedToken(null);
  };

  const handleConfirmPurchase = (shares: number) => {
    // 实现实际的购买逻辑
    console.log(`购买 ${selectedToken?.tokenBatchNumber} 的 ${shares} 份额`);
    // 更新可售份额
    if (selectedToken) {
      setTokenData(prevData =>
        prevData.map(item =>
          item.tokenBatchNumber === selectedToken.tokenBatchNumber
            ? { ...item, availableShares: item.availableShares - shares, soldShares: item.soldShares + shares }
            : item
        )
      );
    }
  };

  return (
    <Box>
      <TableContainer component={Paper}>
        <Table aria-label="token list">
          <TableHead>
            <TableRow>
              <TableCell>token批次编号</TableCell>
              <TableCell>债权人</TableCell>
              <TableCell>债务人</TableCell>
              <TableCell>稳定币</TableCell>
              <TableCell>发行份额</TableCell>
              <TableCell>累计总份额</TableCell>
              <TableCell>有效总份额</TableCell>
              <TableCell>已售份额</TableCell>
              <TableCell>可售份额</TableCell>
              <TableCell>累计偿还份额</TableCell>
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
                      handleViewDetails(row);
                    }}
                  >
                    {row.tokenBatchNumber}
                  </Link>
                </TableCell>
                <TableCell>{row.creditor}</TableCell>
                <TableCell>{row.debtor}</TableCell>
                <TableCell>{row.stablecoin}</TableCell>
                <TableCell>{row.issuedShares}</TableCell>
                <TableCell>{row.totalAccumulatedShares}</TableCell>
                <TableCell>{row.totalValidShares}</TableCell>
                <TableCell>{row.soldShares}</TableCell>
                <TableCell>{row.availableShares}</TableCell>
                <TableCell>{row.totalRepaidShares}</TableCell>
                <TableCell>
                  <Button
                    variant="contained"
                    onClick={() => handleBuy(row)}
                    disabled={row.availableShares === 0}
                  >
                    购买
                  </Button>
                  <Tooltip title="详情">
                    <Button
                      variant="outlined"
                      onClick={() => handleViewDetails(row)}
                      sx={{ ml: 1 }}
                    >
                      <Visibility />
                    </Button>
                  </Tooltip>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
      <Box sx={{ display: 'flex', justifyContent: 'center', mt: 2 }}>
        <Pagination
          count={Math.ceil(tokenData.length / rowsPerPage)}
          page={page}
          onChange={handleChangePage}
          variant="outlined"
          shape="rounded"
        />
      </Box>
      <Dialog 
        open={openPurchaseDialog} 
        onClose={handleClosePurchaseDialog}
        sx={{ '& .MuiDialog-paper': {minWidth:'400px', maxWidth: '800px' } }} 
      >
        <DialogTitle>购买 Token</DialogTitle>
        {selectedToken && (
          <TokenPurchaseForm
            token={selectedToken}
            onPurchase={handleConfirmPurchase}
            onClose={handleClosePurchaseDialog}
          />
        )}
      </Dialog>
    </Box>
  );
};

export default TokenList;
