import React, { useState, useEffect } from 'react';
import { 
  Box,
  Card,
  CardContent,
  Button,
  Typography,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Grid,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  CircularProgress,
  SelectChangeEvent
} from '@mui/material';
import TokenService from '../services/tokenService';
import InvoiceService from '../services/invoiceService';

// Helper functions for formatting
const formatAmount = (amount: number, decimals: number = 2): string => {
  return amount.toLocaleString('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
};

const formatDate = (date: string): string => {
  const d = new Date(date);
  if (isNaN(d.getTime())) return 'Invalid Date';
  return d.toLocaleDateString();
};

interface InvoiceBatchDetailProps {
  batchId: string;
}

interface InvoiceBatch {
  id: string;
  creditor_name: string;
  debtor_name: string;
  accepted_currency: string;
  status: string;
  created_at: string;
  total_amount?: string;
}

interface Invoice {
  id: string;
  invoice_number: string;
  amount: number;
  due_date: string;
  payee: string;
  payer: string;
  currency: string;
}

const InvoiceBatchDetail: React.FC<InvoiceBatchDetailProps> = ({ batchId }) => {
  const [batchDetail, setBatchDetail] = useState<InvoiceBatch | null>(null);
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [formValues, setFormValues] = useState({
    batch_reference: '',
    stablecoin_symbol: 'USDC',
    token_value: '1.00',
    interest_rate_apy: '5.00',
    maturity_date: '' as string,
  });
  
  const tokenService = TokenService.getInstance();
  const invoiceService = InvoiceService.getInstance();
  
  useEffect(() => {
    const loadBatchDetails = async () => {
      try {
        setLoading(true);
        const batchResponse = await invoiceService.getInvoiceBatchById(batchId);
        setBatchDetail(batchResponse);
        
        const invoicesResponse = await invoiceService.getInvoicesByBatchId(batchId);
        setInvoices(invoicesResponse);
        
        setFormValues(prev => ({
          ...prev,
          stablecoin_symbol: batchResponse.accepted_currency,
          maturity_date: ''
        }));
      } catch (error) {
        console.error('加载批次详情失败:', error);
        alert('无法加载批次详情');
      } finally {
        setLoading(false);
      }
    };
    
    loadBatchDetails();
  }, [batchId]);
  
  const handleCreateTokenBatch = () => {
    setFormValues(prev => ({
      ...prev,
      batch_reference: `BATCH-${batchDetail?.id.substring(0, 6) || 'NEW'}`,
      maturity_date: ''
    }));
    setModalOpen(true);
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
  
  const handleSelectChange = (event: SelectChangeEvent<string>) => {
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
    try {
      setCreating(true);
      
      const formattedDate = formValues.maturity_date 
        ? new Date(formValues.maturity_date).toISOString()
        : undefined;
      
      await tokenService.createTokenBatchFromInvoiceBatch(batchId, {
        batch_reference: formValues.batch_reference,
        stablecoin_symbol: formValues.stablecoin_symbol,
        token_value: formValues.token_value,
        interest_rate_apy: formValues.interest_rate_apy,
        maturity_date: formattedDate
      });
      
      alert('Token批次创建成功');
      setModalOpen(false);
      
      const updatedBatch = await invoiceService.getInvoiceBatchById(batchId);
      setBatchDetail(updatedBatch);
    } catch (error) {
      console.error('创建Token批次失败:', error);
      alert('创建Token批次失败');
    } finally {
      setCreating(false);
    }
  };
  
  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', p: 3 }}>
        <CircularProgress />
      </Box>
    );
  }
  
  if (!batchDetail) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography variant="h5">未找到批次信息</Typography>
      </Box>
    );
  }
  
  const totalAmount = invoices.reduce((sum, invoice) => sum + invoice.amount, 0);
  
  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" gutterBottom>
        发票批次详情
      </Typography>
      
      <Paper elevation={2} sx={{ mb: 4 }}>
        <Box sx={{ p: 3 }}>
          <Typography variant="h6" gutterBottom>批次基本信息</Typography>
          <Grid container spacing={2}>
            <Grid item xs={12} md={6}>
              <Box sx={{ mb: 2 }}>
                <Typography variant="subtitle2" color="textSecondary">批次ID</Typography>
                <Typography variant="body1">{batchDetail.id}</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} md={6}>
              <Box sx={{ mb: 2 }}>
                <Typography variant="subtitle2" color="textSecondary">债权人</Typography>
                <Typography variant="body1">{batchDetail.creditor_name}</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} md={6}>
              <Box sx={{ mb: 2 }}>
                <Typography variant="subtitle2" color="textSecondary">债务人</Typography>
                <Typography variant="body1">{batchDetail.debtor_name}</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} md={6}>
              <Box sx={{ mb: 2 }}>
                <Typography variant="subtitle2" color="textSecondary">接受币种</Typography>
                <Typography variant="body1">{batchDetail.accepted_currency}</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} md={6}>
              <Box sx={{ mb: 2 }}>
                <Typography variant="subtitle2" color="textSecondary">状态</Typography>
                <Typography variant="body1">{batchDetail.status}</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} md={6}>
              <Box sx={{ mb: 2 }}>
                <Typography variant="subtitle2" color="textSecondary">创建时间</Typography>
                <Typography variant="body1">{formatDate(batchDetail.created_at)}</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} md={6}>
              <Box sx={{ mb: 2 }}>
                <Typography variant="subtitle2" color="textSecondary">发票总数</Typography>
                <Typography variant="body1">{invoices.length}</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} md={6}>
              <Box sx={{ mb: 2 }}>
                <Typography variant="subtitle2" color="textSecondary">总金额</Typography>
                <Typography variant="body1">{formatAmount(totalAmount)} {batchDetail.accepted_currency}</Typography>
              </Box>
            </Grid>
          </Grid>
        </Box>
      </Paper>
      
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
        <Typography variant="h5">包含的发票</Typography>
        {batchDetail.status === 'Issued' && (
          <Button 
            variant="contained" 
            color="primary" 
            onClick={handleCreateTokenBatch}
          >
            创建Token批次
          </Button>
        )}
      </Box>
      
      {invoices.length > 0 ? (
        <TableContainer component={Paper}>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>发票编号</TableCell>
                <TableCell>金额</TableCell>
                <TableCell>到期日</TableCell>
                <TableCell>债权人</TableCell>
                <TableCell>债务人</TableCell>
                <TableCell>币种</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {invoices.map(invoice => (
                <TableRow key={invoice.id}>
                  <TableCell>{invoice.invoice_number}</TableCell>
                  <TableCell>{formatAmount(invoice.amount)}</TableCell>
                  <TableCell>{formatDate(invoice.due_date)}</TableCell>
                  <TableCell>{invoice.payee}</TableCell>
                  <TableCell>{invoice.payer}</TableCell>
                  <TableCell>{invoice.currency}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      ) : (
        <Paper sx={{ p: 3, textAlign: 'center' }}>
          <Typography color="textSecondary">该批次暂无发票</Typography>
        </Paper>
      )}
      
      <Dialog 
        open={modalOpen} 
        onClose={handleModalClose}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>创建Token批次</DialogTitle>
        <DialogContent>
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

export default InvoiceBatchDetail; 