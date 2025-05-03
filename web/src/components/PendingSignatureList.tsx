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
  Link,
  Pagination,
  Select,
  MenuItem,
  InputLabel,
  FormControl,
  SelectChangeEvent,
  Tooltip,
  IconButton,
} from '@mui/material';
import HistoryIcon from '@mui/icons-material/History';
import Layout from './Layout';

// 定义待签名数据类型，新增 createDate 字段
type PendingSignatureData = {
  tokenBatchNumber: string;
  creditorAccount: string;
  debtor: string;
  stablecoin: string;
  billQuantity: number;
  issuedAmount: bigint;
  createDate: string;
};

// 定义 PendingSignatureList 组件的 Props 类型
type PendingSignatureListProps = {
  onSign: (data: PendingSignatureData) => void;
};

const PendingSignatureList: React.FC<PendingSignatureListProps> = ({ onSign }) => {
  const [pendingSignatureData, setPendingSignatureData] = useState<PendingSignatureData[]>([]);
  const [page, setPage] = useState(1);
  const [rowsPerPage, setRowsPerPage] = useState(10);
  const rowsPerPageOptions = [5, 10, 20];

  // 模拟获取 20 条待签名数据
  useEffect(() => {
    const mockData: PendingSignatureData[] = Array.from({ length: 20 }, (_, index) => ({
      tokenBatchNumber: `Batch-${String(index + 1).padStart(3, '0')}`,
      creditorAccount: `0x${Math.random().toString(16).substr(2, 40)}`,
      debtor: `0x${Math.random().toString(16).substr(2, 40)}`,
      stablecoin: index % 2 === 0 ? 'USDT' : 'MNT',
      billQuantity: Math.floor(Math.random() * 50) + 1,
      issuedAmount: BigInt(Math.floor(Math.random() * 1000000) + 100000),
      createDate: new Date(Date.now() - index * 86400000).toISOString().split('T')[0],
    }));
    setPendingSignatureData(mockData);
  }, []);

  const handleChangePage = (event: React.ChangeEvent<unknown>, newPage: number) => {
    setPage(newPage);
  };

  const handleChangeRowsPerPage = (event: SelectChangeEvent<number>) => {
    setRowsPerPage(Number(event.target.value));
    setPage(1); // 当改变每页行数时，重置页码为 1
  };

  const indexOfLastRow = page * rowsPerPage;
  const indexOfFirstRow = indexOfLastRow - rowsPerPage;
  const currentRows = pendingSignatureData.slice(indexOfFirstRow, indexOfLastRow);

  return (
    <Layout>
    <Box>
      <TableContainer component={Paper}>
        <Table aria-label="pending signature list">
          <TableHead>
            <TableRow>
              <TableCell>token批次编号</TableCell>
              <TableCell>债权人账户</TableCell>
              <TableCell>债务方</TableCell>
              <TableCell>稳定币</TableCell>
              <TableCell>票据数量</TableCell>
              <TableCell>发行币额</TableCell>
              <TableCell>创建日期</TableCell>
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
                      // 可在这里添加点击超链接后的逻辑
                      console.log(`点击了 ${row.tokenBatchNumber} 超链接`);
                    }}
                  >
                    {row.tokenBatchNumber}
                  </Link>
                </TableCell>
                <TableCell>{row.creditorAccount}</TableCell>
                <TableCell>{row.debtor}</TableCell>
                <TableCell>{row.stablecoin}</TableCell>
                <TableCell>{row.billQuantity}</TableCell>
                <TableCell>
                  <Typography>{row.issuedAmount.toString()}</Typography>
                </TableCell>
                <TableCell>{row.createDate}</TableCell>
                <TableCell>
                  <Button variant="contained" onClick={() => onSign(row)}>
                    签名
                  </Button>

                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', mt: 2, alignItems: 'center' }}>
        <FormControl sx={{ minWidth: 70 }}>
          {/* <InputLabel>每页条数</InputLabel> */}
          <Select
            value={rowsPerPage}
            onChange={handleChangeRowsPerPage}
            label="每页条数"
          >
            {rowsPerPageOptions.map((option) => (
              <MenuItem key={option} value={option}>
                {option}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
        <Pagination
          count={Math.ceil(pendingSignatureData.length / rowsPerPage)}
          page={page}
          onChange={handleChangePage}
          variant="outlined"
          shape="rounded"
        />
      </Box>
    </Box>
    </Layout>
  );
};

export default PendingSignatureList;
