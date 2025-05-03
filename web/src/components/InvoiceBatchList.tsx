import React, { useState, useEffect } from 'react';
import {
  Box,
  Button,
  Paper,
  Typography,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TablePagination,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Card,
  CardContent,
  Chip,
  IconButton,
  Grid,
  SelectChangeEvent
} from '@mui/material';
import { Visibility, ShoppingCart } from '@mui/icons-material';
import { useNavigate } from 'react-router-dom';
import InvoiceService from '../services/invoiceService';
import TokenService from '../services/tokenService';
import { formatAmount, formatDate, formatStatus } from '../utils/formatters';

interface InvoiceBatch {
  id: string;
  creditor_name: string;
  debtor_name: string;
  accepted_currency: string;
  status: string;
  created_at: string;
  invoice_count?: number;
  total_amount?: number;
}

interface BatchSummary {
  total_amount: number;
  earliest_date: string;
  latest_date: string;
  invoice_count: number;
}

interface Invoice {
  id: string;
  invoice_number: string;
  amount: number;
  due_date: string;
  currency: string;
}

const InvoiceBatchList: React.FC = () => {
  const [batches, setBatches] = useState<InvoiceBatch[]>([]);
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [creating, setCreating] = useState(false);
  const [currentBatch, setCurrentBatch] = useState<InvoiceBatch | null>(null);
  const [batchSummary, setBatchSummary] = useState<BatchSummary | null>(null);
  const [page, setPage] = useState(0);
  const [rowsPerPage, setRowsPerPage] = useState(10);
  const [formValues, setFormValues] = useState({
    batch_reference: '',
    stablecoin_symbol: 'USDC',
    token_value: '1.00',
    interest_rate_apy: '5.00',
    maturity_date: '' as string,
  });
  
  const navigate = useNavigate();
  
  const invoiceService = InvoiceService.getInstance();
  const tokenService = TokenService.getInstance();
  
  useEffect(() => {
    fetchBatches();
  }, []);
  
  const fetchBatches = async () => {
    try {
      setLoading(true);
      const response = await invoiceService.getUserInvoiceBatches();
      setBatches(response);
    } catch (error) {
      console.error('获取发票批次失败:', error);
      // Show error notification
    } finally {
      setLoading(false);
    }
  };
  
  const handleViewDetail = (batchId: string) => {
    navigate(`/invoice-batch/${batchId}`);
  };
  
  const handleListToken = async (batch: InvoiceBatch) => {
    try {
      // 获取批次详情和摘要信息
      const invoices = await invoiceService.getInvoicesByBatchId(batch.id);
      
      // 计算摘要信息
      const total = invoices.reduce((sum: number, inv: Invoice) => sum + inv.amount, 0);
      let earliest = new Date().toISOString();
      let latest = new Date(0).toISOString();
      
      invoices.forEach((inv: Invoice) => {
        if (inv.due_date < earliest) earliest = inv.due_date;
        if (inv.due_date > latest) latest = inv.due_date;
      });
      
      setBatchSummary({
        total_amount: total,
        earliest_date: earliest,
        latest_date: latest,
        invoice_count: invoices.length
      });
      
      setCurrentBatch(batch);
      setFormValues({
        batch_reference: `BATCH-${batch.id.substring(0, 6)}`,
        stablecoin_symbol: batch.accepted_currency,
        token_value: "1.00",
        interest_rate_apy: "5.00",
        maturity_date: ''
      });
      setModalOpen(true);
    } catch (error) {
      console.error('获取批次详情失败:', error);
      // Show error notification
    }
  };
  
  const handleModalClose = () => {
    setModalOpen(false);
  };
  
  const handleInputChange = (event: React.ChangeEvent<HTMLInputElement | { name?: string; value: unknown }>) => {
    const { name, value } = event.target;
    if (name) {
      setFormValues({
        ...formValues,
        [name]: value
      });
    }
  };
  
  const handleSelectChange = (event: SelectChangeEvent) => {
    const { name, value } = event.target;
    if (name) {
      setFormValues({
        ...formValues,
        [name]: value
      });
    }
  };
  
  const handleDateChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setFormValues({
      ...formValues,
      maturity_date: event.target.value
    });
  };
  
  const handleModalSubmit = async () => {
    if (!currentBatch) return;
    
    try {
      setCreating(true);
      
      // Format the date string to ISO string if present
      const formattedDate = formValues.maturity_date 
        ? new Date(formValues.maturity_date).toISOString()
        : undefined;
        
      await tokenService.createTokenBatchFromInvoiceBatch(
        currentBatch.id, 
        {
          batch_reference: formValues.batch_reference,
          stablecoin_symbol: formValues.stablecoin_symbol,
          token_value: formValues.token_value,
          interest_rate_apy: formValues.interest_rate_apy,
          maturity_date: formattedDate
        }
      );
      
      // Show success notification
      alert('Token批次创建成功');
      setModalOpen(false);
      
      // 刷新批次列表
      fetchBatches();
    } catch (error) {
      console.error('创建Token批次失败:', error);
      // Show error notification
      alert('创建Token批次失败');
    } finally {
      setCreating(false);
    }
  };
  
  const getStatusChip = (status: string) => {
    let color = 'default';
    switch (status) {
      case 'Packaging':
        color = 'primary';
        break;
      case 'Issued':
        color = 'success';
        break;
      case 'Trading':
        color = 'warning';
        break;
      case 'Repaying':
        color = 'info';
        break;
      case 'Settled':
        color = 'success';
        break;
      case 'Defaulted':
        color = 'error';
        break;
    }
    return <Chip label={formatStatus(status)} color={color as any} size="small" />;
  };
  
  const handleChangePage = (event: unknown, newPage: number) => {
    setPage(newPage);
  };

  const handleChangeRowsPerPage = (event: React.ChangeEvent<HTMLInputElement>) => {
    setRowsPerPage(parseInt(event.target.value, 10));
    setPage(0);
  };
  
  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" gutterBottom>
        发票批次管理
      </Typography>
      
      <Paper sx={{ width: '100%', mb: 2 }}>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>批次ID</TableCell>
                <TableCell>债权人</TableCell>
                <TableCell>债务人</TableCell>
                <TableCell>接受币种</TableCell>
                <TableCell>状态</TableCell>
                <TableCell>发票数量</TableCell>
                <TableCell>总金额</TableCell>
                <TableCell>创建时间</TableCell>
                <TableCell>操作</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {batches
                .slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage)
                .map((batch) => (
                <TableRow key={batch.id}>
                  <TableCell>{batch.id.substring(0, 8)}...</TableCell>
                  <TableCell>{batch.creditor_name}</TableCell>
                  <TableCell>{batch.debtor_name}</TableCell>
                  <TableCell>{batch.accepted_currency}</TableCell>
                  <TableCell>{getStatusChip(batch.status)}</TableCell>
                  <TableCell>{batch.invoice_count || 0}</TableCell>
                  <TableCell>
                    {batch.total_amount 
                      ? `${formatAmount(parseFloat(batch.total_amount))} ${batch.accepted_currency}` 
                      : '-'}
                  </TableCell>
                  <TableCell>{formatDate(batch.created_at)}</TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex' }}>
                      <IconButton 
                        size="small" 
                        color="primary" 
                        onClick={() => handleViewDetail(batch.id)}
                      >
                        <Visibility />
                      </IconButton>
                      {batch.status === 'Issued' && (
                        <IconButton 
                          size="small" 
                          color="success" 
                          onClick={() => handleListToken(batch)}
                        >
                          <ShoppingCart />
                        </IconButton>
                      )}
                    </Box>
                  </TableCell>
                </TableRow>
              ))}
              {loading && (
                <TableRow>
                  <TableCell colSpan={9} align="center">
                    加载中...
                  </TableCell>
                </TableRow>
              )}
              {!loading && batches.length === 0 && (
                <TableRow>
                  <TableCell colSpan={9} align="center">
                    暂无数据
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </TableContainer>
        <TablePagination
          rowsPerPageOptions={[5, 10, 25]}
          component="div"
          count={batches.length}
          rowsPerPage={rowsPerPage}
          page={page}
          onPageChange={handleChangePage}
          onRowsPerPageChange={handleChangeRowsPerPage}
          labelRowsPerPage="每页行数:"
        />
      </Paper>
      
      <Dialog 
        open={modalOpen} 
        onClose={handleModalClose}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>创建Token批次</DialogTitle>
        <DialogContent>
          {batchSummary && (
            <Card sx={{ mb: 2, mt: 1 }}>
              <CardContent>
                <Typography variant="h6" gutterBottom>批次信息摘要</Typography>
                <Grid container spacing={2}>
                  <Grid item xs={6}>
                    <Typography variant="body1">
                      <strong>票据总额:</strong> {formatAmount(batchSummary.total_amount)} {currentBatch?.accepted_currency}
                    </Typography>
                  </Grid>
                  <Grid item xs={6}>
                    <Typography variant="body1">
                      <strong>票据数量:</strong> {batchSummary.invoice_count}
                    </Typography>
                  </Grid>
                  <Grid item xs={6}>
                    <Typography variant="body1">
                      <strong>最早到期日:</strong> {formatDate(batchSummary.earliest_date)}
                    </Typography>
                  </Grid>
                  <Grid item xs={6}>
                    <Typography variant="body1">
                      <strong>最晚到期日:</strong> {formatDate(batchSummary.latest_date)}
                    </Typography>
                  </Grid>
                </Grid>
              </CardContent>
            </Card>
          )}
          
          <Box component="form" sx={{ mt: 2 }}>
            <TextField
              fullWidth
              margin="normal"
              name="batch_reference"
              label="批次标识"
              value={formValues.batch_reference}
              onChange={handleInputChange}
              required
              placeholder="例如: BATCH-001"
            />
            
            <FormControl fullWidth margin="normal">
              <InputLabel id="stablecoin-label">稳定币符号</InputLabel>
              <Select
                labelId="stablecoin-label"
                name="stablecoin_symbol"
                value={formValues.stablecoin_symbol}
                onChange={handleSelectChange}
                label="稳定币符号"
              >
                <MenuItem value="USDC">USDC</MenuItem>
                <MenuItem value="USDT">USDT</MenuItem>
                <MenuItem value="DAI">DAI</MenuItem>
              </Select>
            </FormControl>
            
            <TextField
              fullWidth
              margin="normal"
              name="token_value"
              label="代币单价"
              value={formValues.token_value}
              onChange={handleInputChange}
              required
              placeholder="例如: 1.00"
            />
            
            <TextField
              fullWidth
              margin="normal"
              name="interest_rate_apy"
              label="年化利率(%)"
              value={formValues.interest_rate_apy}
              onChange={handleInputChange}
              required
              placeholder="例如: 5.00"
            />
            
            <TextField
              fullWidth
              margin="normal"
              name="maturity_date"
              label="到期日期(可选，默认使用最早发票到期日)"
              type="date"
              value={formValues.maturity_date}
              onChange={handleDateChange}
              InputLabelProps={{
                shrink: true,
              }}
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={handleModalClose}>取消</Button>
          <Button 
            onClick={handleModalSubmit} 
            variant="contained" 
            color="primary"
            disabled={creating}
          >
            {creating ? '创建中...' : '创建'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default InvoiceBatchList; 