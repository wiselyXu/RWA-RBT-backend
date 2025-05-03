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
  Typography,
  Button,
  Tooltip,
  IconButton,
} from '@mui/material';
import HistoryIcon from '@mui/icons-material/History';
import VisibilityIcon from '@mui/icons-material/Visibility';

// 定义 Token 数据类型
type MyTokenData = {
  walletAddress: string;
  tokenBatchNumber: string;
  stableCoin: string;
  totalHoldingShares: bigint;
  purchaseShares: bigint;
  interestShares: bigint;
  effectiveShares: bigint;
  repaymentAllocation: bigint;
  bookShares: bigint;
  interestTransferred: bigint;
  interestUnTransferred: bigint;
  repaymentAllocationSettled: bigint;
  repaymentAllocationUnsettled: bigint;
  eventType: string;
  createTime: Date;
  createWallet: string;
  modifyTime: Date;
};

// 定义 MyTokenList 组件的 Props 类型
// type MyTokenListProps = {
//   onAdditionalPurchase: (data: MyTokenData) => void;
//   onViewBillHistory: (data: MyTokenData) => void;
//   onViewRepayDebtOverview: (tokenBatchNumber: string) => void;
// };

const MyTokenList: React.FC= () => {
  const [myTokenData, setMyTokenData] = useState<MyTokenData[]>([]);

  // 模拟获取 Token 数据
  useEffect(() => {
    const mockData: MyTokenData[] = Array.from({ length: 5 }, (_, index) => {
      const purchaseShares = BigInt(Math.floor(Math.random() * 10000));
      const interestShares = BigInt(Math.floor(Math.random() * 1000));
      const repaymentAllocation = BigInt(Math.floor(Math.random() * 500));
      const interestTransferred = BigInt(Math.floor(Math.random() * Number(interestShares)));
      const repaymentAllocationSettled = BigInt(Math.floor(Math.random() * Number(repaymentAllocation)));
      return {
        walletAddress: `0x${Math.random().toString(16).substr(2, 40)}`,
        tokenBatchNumber: `Batch-${String(index + 1).padStart(3, '0')}`,
        stableCoin: index % 2 === 0 ? 'USDT' : 'MNT',
        totalHoldingShares: purchaseShares + interestShares,
        purchaseShares,
        interestShares,
        effectiveShares: purchaseShares + interestShares - repaymentAllocation,
        repaymentAllocation,
        bookShares: purchaseShares + interestShares - repaymentAllocation + BigInt(Math.floor(Math.random() * 200) - 100),
        interestTransferred,
        interestUnTransferred: interestShares - interestTransferred,
        repaymentAllocationSettled,
        repaymentAllocationUnsettled: repaymentAllocation - repaymentAllocationSettled,
        eventType: ['interest', 'purchase', 'repay', 'claim interest', 'claim repay'][index % 5],
        createTime: new Date(Date.now() - index * 86400000),
        createWallet: `Wallet-${String(index + 1).padStart(3, '0')}`,
        modifyTime: new Date(Date.now() - (index % 2) * 43200000),
      };
    });
    setMyTokenData(mockData);
  }, []);

    function onAdditionalPurchase(row: MyTokenData): void {
        throw new Error('Function not implemented.');
    }

    function onViewBillHistory(row: MyTokenData): void {
        throw new Error('Function not implemented.');
    }

    function onViewRepayDebtOverview(tokenBatchNumber: string): void {
        throw new Error('Function not implemented.');
    }

  return (
    <Box sx={{ 
      overflowX: 'auto', 
      overflowY: 'auto',
      // 确保 Box 组件宽度可以根据内容自适应
      width: '100%', 
      // 限制最大高度，超过该高度会出现垂直滚动条
      maxHeight: 600 
    }}>
      <TableContainer component={Paper}>
        <Table aria-label="my token list" sx={{ 
          // 确保表格宽度可以根据内容自适应
          minWidth: 1200 
        }}>
          <TableHead>
            <TableRow>
              <TableCell>钱包地址</TableCell>
              <TableCell>token批次编号</TableCell>
              <TableCell>稳定币种</TableCell>
              <TableCell>持有总份额</TableCell>
              <TableCell>购买份额</TableCell>
              <TableCell>利息份额</TableCell>
              <TableCell>有效份额</TableCell>
              <TableCell>偿还分摊</TableCell>
              <TableCell>账面份额</TableCell>
              <TableCell>利息已转</TableCell>
              <TableCell>利息未转</TableCell>
              <TableCell>偿还分摊已结</TableCell>
              <TableCell>偿还分摊未结</TableCell>
              <TableCell>事件类型</TableCell>
              <TableCell>创建时间</TableCell>
              <TableCell>创建钱包</TableCell>
              <TableCell>修改时间</TableCell>
              <TableCell>操作</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {myTokenData.map((row, index) => (
              <TableRow key={index}>
                <TableCell>{row.walletAddress}</TableCell>
                <TableCell>{row.tokenBatchNumber}</TableCell>
                <TableCell>{row.stableCoin}</TableCell>
                <TableCell>{row.totalHoldingShares.toString()}</TableCell>
                <TableCell>{row.purchaseShares.toString()}</TableCell>
                <TableCell>{row.interestShares.toString()}</TableCell>
                <TableCell>{row.effectiveShares.toString()}</TableCell>
                <TableCell>{row.repaymentAllocation.toString()}</TableCell>
                <TableCell>{row.bookShares.toString()}</TableCell>
                <TableCell>{row.interestTransferred.toString()}</TableCell>
                <TableCell>{row.interestUnTransferred.toString()}</TableCell>
                <TableCell>{row.repaymentAllocationSettled.toString()}</TableCell>
                <TableCell>{row.repaymentAllocationUnsettled.toString()}</TableCell>
                <TableCell>{row.eventType}</TableCell>
                <TableCell>{row.createTime.toLocaleDateString()}</TableCell>
                <TableCell>{row.createWallet}</TableCell>
                <TableCell>{row.modifyTime.toLocaleDateString()}</TableCell>
                <TableCell>
                  <Button variant="contained" size="small" onClick={() => onAdditionalPurchase(row)}>
                    增购
                  </Button>
                  <Tooltip title="查看账单历史">
                    <IconButton onClick={() => onViewBillHistory(row)}>
                      <HistoryIcon />
                    </IconButton>
                  </Tooltip>
                  <Tooltip title="查看还款概览">
                    <IconButton onClick={() => onViewRepayDebtOverview(row.tokenBatchNumber)}>
                      <VisibilityIcon />
                    </IconButton>
                  </Tooltip>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </Box>
  );
};

export default MyTokenList;
