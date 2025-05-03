import React from 'react';
import {
  Box,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Typography,
  Grid,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  TextField,
  InputAdornment,
  Chip,
} from '@mui/material';

interface Bill {
  id: string;
  creditorAccount: string;
  debtor: string;
  amount: number;
  billImageUrl: string;
  contractImageUrl: string;
  tokenBatchCode: string;
  isCleared: boolean;
  isOnChain: boolean;
  createdAt: Date;
  createdWallet: string;
  updatedAt: Date;
  updatedWallet: string;
  dueDate: string;
}

interface TokenizationForm {
  stableCoin: string;
  minTerm: number;
  maxTerm: number;
  interestRate: number;
  defaultRate: number;
}

interface TokenizationDialogProps {
  open: boolean;
  onClose: () => void;
  onSubmit: () => void;
  selectedBillsForTokenization: Bill[];
  tokenizationForm: TokenizationForm;
  handleTokenizationFormChange: (field: keyof TokenizationForm, value: string | number) => void;
}

const TokenizationDialog: React.FC<TokenizationDialogProps> = ({
  open,
  onClose,
  onSubmit,
  selectedBillsForTokenization,
  tokenizationForm,
  handleTokenizationFormChange,
}) => {
  const totalAmount = selectedBillsForTokenization.reduce((sum, bill) => sum + bill.amount, 0);

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="md"
      fullWidth
    >
      <DialogTitle>票据打包货币化</DialogTitle>
      <DialogContent>
        <Box sx={{ pt: 2 }}>
          <Typography variant="h6" gutterBottom>
            已选票据列表
          </Typography>
          <TableContainer component={Paper} sx={{ mb: 3 }}>
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>票据编号</TableCell>
                  <TableCell>债务人</TableCell>
                  <TableCell>金额</TableCell>
                  <TableCell>状态</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {selectedBillsForTokenization.map((bill) => (
                  <TableRow key={bill.id}>
                    <TableCell>{bill.id}</TableCell>
                    <TableCell>{bill.debtor}</TableCell>
                    <TableCell>{bill.amount.toLocaleString()} 元</TableCell>
                    <TableCell>
                      <Chip
                        label={bill.isOnChain ? '已上链' : '未上链'}
                        color={bill.isOnChain ? 'success' : 'default'}
                        size="small"
                      />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>

          <Typography variant="h6" gutterBottom>
            总金额: {totalAmount.toLocaleString()} 元
          </Typography>

          <Grid container spacing={2} sx={{ mt: 2 }}>
            <Grid item xs={12} sm={6}>
              <FormControl fullWidth>
                <InputLabel>稳定币种</InputLabel>
                <Select
                  value={tokenizationForm.stableCoin}
                  label="稳定币种"
                  onChange={(e) => handleTokenizationFormChange('stableCoin', e.target.value)}
                >
                  <MenuItem value="USDT">USDT</MenuItem>
                  <MenuItem value="USDC">USDC</MenuItem>
                  <MenuItem value="DAI">DAI</MenuItem>
                </Select>
              </FormControl>
            </Grid>
            <Grid item xs={12} sm={6}>
              <TextField
                fullWidth
                label="最短期限（月）"
                type="number"
                value={tokenizationForm.minTerm}
                onChange={(e) => handleTokenizationFormChange('minTerm', Number(e.target.value))}
                InputProps={{
                  endAdornment: <InputAdornment position="end">月</InputAdornment>,
                }}
              />
            </Grid>
            <Grid item xs={12} sm={6}>
              <TextField
                fullWidth
                label="最晚期限（月）"
                type="number"
                value={tokenizationForm.maxTerm}
                onChange={(e) => handleTokenizationFormChange('maxTerm', Number(e.target.value))}
                InputProps={{
                  endAdornment: <InputAdornment position="end">月</InputAdornment>,
                }}
              />
            </Grid>
            <Grid item xs={12} sm={6}>
              <TextField
                fullWidth
                label="利率"
                type="number"
                value={tokenizationForm.interestRate}
                onChange={(e) => handleTokenizationFormChange('interestRate', Number(e.target.value))}
                InputProps={{
                  endAdornment: <InputAdornment position="end">%</InputAdornment>,
                }}
              />
            </Grid>
            <Grid item xs={12} sm={6}>
              <TextField
                fullWidth
                label="违约利率"
                type="number"
                value={tokenizationForm.defaultRate}
                onChange={(e) => handleTokenizationFormChange('defaultRate', Number(e.target.value))}
                InputProps={{
                  endAdornment: <InputAdornment position="end">%</InputAdornment>,
                }}
              />
            </Grid>
          </Grid>
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>取消</Button>
        <Button onClick={onSubmit} variant="contained" color="primary">
          确认打包
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default TokenizationDialog;
