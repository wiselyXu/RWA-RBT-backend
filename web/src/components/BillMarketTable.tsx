import React, { useState } from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Button,
  Typography,
  TablePagination,
  TextField,
  Toolbar,
  Box,
  Snackbar,
  Alert,
} from '@mui/material';

interface BillMarketRow {
  creditor: string;
  debtor: string;
  batch: string;
  issueDate: string;
  totalAmount: number;
  soldAmount: number;
  repaidAmount: number;
  availableAmount: number;
  lastRepayDate: string;
  firstRepayDate: string;
}

const mockData: BillMarketRow[] = [
  { creditor: '企业A', debtor: '企业B', batch: 'RBT202401', issueDate: '2024-04-01', totalAmount: 1000000, soldAmount: 600000, repaidAmount: 200000, availableAmount: 400000, lastRepayDate: '2025-04-01', firstRepayDate: '2024-10-01', },
  { creditor: '企业C', debtor: '企业D', batch: 'RBT202402', issueDate: '2024-05-01', totalAmount: 500000, soldAmount: 200000, repaidAmount: 100000, availableAmount: 300000, lastRepayDate: '2025-05-01', firstRepayDate: '2024-11-01', },
  { creditor: '企业E', debtor: '企业F', batch: 'RBT202403', issueDate: '2024-06-01', totalAmount: 800000, soldAmount: 400000, repaidAmount: 200000, availableAmount: 400000, lastRepayDate: '2025-06-01', firstRepayDate: '2024-12-01', },
  { creditor: '企业G', debtor: '企业H', batch: 'RBT202404', issueDate: '2024-07-01', totalAmount: 1200000, soldAmount: 700000, repaidAmount: 300000, availableAmount: 500000, lastRepayDate: '2025-07-01', firstRepayDate: '2025-01-01', },
  { creditor: '企业I', debtor: '企业J', batch: 'RBT202405', issueDate: '2024-08-01', totalAmount: 900000, soldAmount: 500000, repaidAmount: 250000, availableAmount: 400000, lastRepayDate: '2025-08-01', firstRepayDate: '2025-02-01', },
  { creditor: '企业K', debtor: '企业L', batch: 'RBT202406', issueDate: '2024-09-01', totalAmount: 1100000, soldAmount: 600000, repaidAmount: 300000, availableAmount: 500000, lastRepayDate: '2025-09-01', firstRepayDate: '2025-03-01', },
  { creditor: '企业M', debtor: '企业N', batch: 'RBT202407', issueDate: '2024-10-01', totalAmount: 950000, soldAmount: 450000, repaidAmount: 200000, availableAmount: 500000, lastRepayDate: '2025-10-01', firstRepayDate: '2025-04-01', },
  { creditor: '企业O', debtor: '企业P', batch: 'RBT202408', issueDate: '2024-11-01', totalAmount: 1050000, soldAmount: 550000, repaidAmount: 250000, availableAmount: 500000, lastRepayDate: '2025-11-01', firstRepayDate: '2025-05-01', },
  { creditor: '企业Q', debtor: '企业R', batch: 'RBT202409', issueDate: '2024-12-01', totalAmount: 1150000, soldAmount: 650000, repaidAmount: 300000, availableAmount: 500000, lastRepayDate: '2025-12-01', firstRepayDate: '2025-06-01', },
  { creditor: '企业S', debtor: '企业T', batch: 'RBT202410', issueDate: '2025-01-01', totalAmount: 1250000, soldAmount: 750000, repaidAmount: 350000, availableAmount: 500000, lastRepayDate: '2026-01-01', firstRepayDate: '2025-07-01', },
];

const BillMarketTable: React.FC = () => {
  const [page, setPage] = useState(0);
  const [rowsPerPage, setRowsPerPage] = useState(5);
  const [searchCreditor, setSearchCreditor] = useState('');
  const [searchDebtor, setSearchDebtor] = useState('');
  const [searchBatch, setSearchBatch] = useState('');
  const [searchLastRepayStart, setSearchLastRepayStart] = useState('');
  const [searchLastRepayEnd, setSearchLastRepayEnd] = useState('');
  const [openSnackbar, setOpenSnackbar] = useState(false);

  const handleChangePage = (event: unknown, newPage: number) => {
    setPage(newPage);
  };

  const handleChangeRowsPerPage = (event: React.ChangeEvent<HTMLInputElement>) => {
    setRowsPerPage(parseInt(event.target.value, 10));
    setPage(0);
  };

  const handleQueryClick = () => {
    setOpenSnackbar(true);
  };

  // 过滤逻辑
  const filteredData = mockData.filter(row => {
    const matchCreditor = row.creditor.includes(searchCreditor);
    const matchDebtor = row.debtor.includes(searchDebtor);
    const matchBatch = row.batch.includes(searchBatch);
    const matchLastRepayStart = searchLastRepayStart ? row.lastRepayDate >= searchLastRepayStart : true;
    const matchLastRepayEnd = searchLastRepayEnd ? row.lastRepayDate <= searchLastRepayEnd : true;
    return matchCreditor && matchDebtor && matchBatch && matchLastRepayStart && matchLastRepayEnd;
  });

  return (
    <Paper sx={{ p: 3, mt: 4 }}>
      <Typography variant="h6" gutterBottom>
        票据市场
      </Typography>
      <Toolbar sx={{ px: 0, mb: 2 }}>
        <Box display="flex" flexWrap="wrap" gap={2} width="100%" alignItems="center">
          <TextField
            label="债权人"
            size="small"
            value={searchCreditor}
            onChange={e => setSearchCreditor(e.target.value)}
          />
          <TextField
            label="债务人"
            size="small"
            value={searchDebtor}
            onChange={e => setSearchDebtor(e.target.value)}
          />
          <TextField
            label="货币批次"
            size="small"
            value={searchBatch}
            onChange={e => setSearchBatch(e.target.value)}
          />
          <TextField
            label="最后偿还日-起"
            size="small"
            type="date"
            InputLabelProps={{ shrink: true }}
            value={searchLastRepayStart}
            onChange={e => setSearchLastRepayStart(e.target.value)}
          />
          <TextField
            label="最后偿还日-止"
            size="small"
            type="date"
            InputLabelProps={{ shrink: true }}
            value={searchLastRepayEnd}
            onChange={e => setSearchLastRepayEnd(e.target.value)}
          />
          <Button variant="contained" color="primary" onClick={handleQueryClick} sx={{ minWidth: 80 }}>
            查询
          </Button>
        </Box>
      </Toolbar>
      <TableContainer>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>债权人</TableCell>
              <TableCell>债务人</TableCell>
              <TableCell>货币批次</TableCell>
              <TableCell>发币日期</TableCell>
              <TableCell>初始发币总额</TableCell>
              <TableCell>已售份额</TableCell>
              <TableCell>已偿还份额</TableCell>
              <TableCell>可售份额</TableCell>
              <TableCell>最后偿还日</TableCell>
              <TableCell>首次可偿还日</TableCell>
              <TableCell>操作</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {filteredData.slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage).map((row, idx) => (
              <TableRow key={idx}>
                <TableCell>{row.creditor}</TableCell>
                <TableCell>{row.debtor}</TableCell>
                <TableCell>{row.batch}</TableCell>
                <TableCell>{row.issueDate}</TableCell>
                <TableCell>{row.totalAmount.toLocaleString()}</TableCell>
                <TableCell>{row.soldAmount.toLocaleString()}</TableCell>
                <TableCell>{row.repaidAmount.toLocaleString()}</TableCell>
                <TableCell>{row.availableAmount.toLocaleString()}</TableCell>
                <TableCell>{row.lastRepayDate}</TableCell>
                <TableCell>{row.firstRepayDate}</TableCell>
                <TableCell>
                  <Button variant="contained" color="primary" size="small">购买</Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
      <TablePagination
        component="div"
        count={filteredData.length}
        page={page}
        onPageChange={handleChangePage}
        rowsPerPage={rowsPerPage}
        onRowsPerPageChange={handleChangeRowsPerPage}
        rowsPerPageOptions={[5, 10]}
        labelRowsPerPage="每页行数"
      />
      <Snackbar open={openSnackbar} autoHideDuration={2000} onClose={() => setOpenSnackbar(false)} anchorOrigin={{ vertical: 'top', horizontal: 'center' }}>
        <Alert severity="info" sx={{ width: '100%' }} onClose={() => setOpenSnackbar(false)}>
          功能待实现
        </Alert>
      </Snackbar>
    </Paper>
  );
};

export default BillMarketTable; 