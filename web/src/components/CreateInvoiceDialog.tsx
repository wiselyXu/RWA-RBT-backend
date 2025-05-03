// @ts-nocheck
// @ts-ignore
import React, { useState } from 'react';
// @ts-ignore
import {
  Box,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Grid,
  TextField,
  CircularProgress,
  Typography,
  InputAdornment,
  Alert,
} from '@mui/material';
import { CloudUpload } from '@mui/icons-material';
import { useAuth } from '../context/AuthContext';
import InvoiceService/*, { CreateInvoiceParams }*/ from '../services/invoiceService'; // Comment out CreateInvoiceParams if modifying payload directly

// Assuming ethers is installed or will be added
// import { ethers } from 'ethers';

interface CreateInvoiceDialogProps {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

// Updated FormData interface
interface FormData {
  payer: string; 
  amount: number;  // 改为number类型
  currency: string; 
  due_date: number | null; // 改为number类型，存储Unix时间戳
  invoice_ipfs_hash: string; 
  contract_ipfs_hash: string; 
}

const CreateInvoiceDialog: React.FC<CreateInvoiceDialogProps> = ({
  open,
  onClose,
  onSuccess,
}) => {
  const { userInfo } = useAuth();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [fileUploading, setFileUploading] = useState(false);
  
  // Updated form state with snake_case
  const [formData, setFormData] = useState<FormData>({
    payer: '',
    amount: 0,  // 默认值改为0
    currency: 'CNY', 
    due_date: null, // 初始值为null
    invoice_ipfs_hash: '', 
    contract_ipfs_hash: '', 
  });

  // Update form data
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    
    // 特殊处理amount字段，将其转换为number类型
    if (name === 'amount') {
      const numericValue = value === '' ? 0 : parseFloat(value);
      setFormData((prev: FormData) => ({
        ...prev,
        [name]: numericValue,
      }));
    } else {
      setFormData((prev: FormData) => ({
        ...prev,
        [name]: value,
      }));
    }
  };

  // Handle date change - 修改为直接存储Unix时间戳
  const handleDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const dateStr = e.target.value; // 格式为 "YYYY-MM-DD"
    console.log('Selected date string:', dateStr);
    
    if (dateStr) {
      // 创建本地日期对象（不使用UTC时区）
      const dateParts = dateStr.split('-');
      const year = parseInt(dateParts[0], 10);
      const month = parseInt(dateParts[1], 10) - 1; // 月份从0开始
      const day = parseInt(dateParts[2], 10);
      
      // 创建特定的日期（设置为当天的午夜时间，本地时区）
      const date = new Date(year, month, day, 0, 0, 0);
      console.log('Parsed date object:', date);
      
      // 转换为Unix时间戳（秒）
      const timestamp = Math.floor(date.getTime() / 1000);
      console.log('Converted timestamp (seconds):', timestamp);
      console.log('Timestamp as date:', new Date(timestamp * 1000).toLocaleString());
      
      setFormData((prev: FormData) => ({
        ...prev,
        due_date: timestamp,
      }));
    } else {
      setFormData((prev: FormData) => ({
        ...prev,
        due_date: null,
      }));
    }
  };

  // Handle file upload (update state keys)
  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>, type: 'invoice' | 'contract') => {
    if (!e.target.files || e.target.files.length === 0) return;
    
    try {
      setFileUploading(true);
      await new Promise(resolve => setTimeout(resolve, 1500)); 
      const mockHash = `mock_${type}_hash_${Math.random().toString(36).substring(2, 10)}`;
      
      // Use snake_case state keys
      if (type === 'invoice') {
        setFormData((prev: FormData) => ({ ...prev, invoice_ipfs_hash: mockHash })); 
      } else {
        setFormData((prev: FormData) => ({ ...prev, contract_ipfs_hash: mockHash })); 
      }
    } catch (err) {
      console.error(`Failed to upload ${type} file:`, err);
      setError(`上传${type === 'invoice' ? '票据' : '合同'}文件失败`);
    } finally {
      setFileUploading(false);
    }
  };

  // Handle form submission
  const handleSubmit = async () => {
    console.log('提交表单数据:', formData);
    console.log('金额类型:', typeof formData.amount);
    console.log('到期日类型:', typeof formData.due_date);
    console.log('到期日值:', formData.due_date);
    console.log('到期日转换后:', formData.due_date ? new Date(formData.due_date * 1000).toLocaleString() : 'null');
    
    // Updated validation with snake_case
    if (!formData.payer || formData.amount <= 0 || !formData.currency || !formData.due_date || !formData.invoice_ipfs_hash || !formData.contract_ipfs_hash) {
      setError('请填写所有必填字段');
      return;
    }
    
    // Validate amount - 修改验证逻辑
    if (formData.amount <= 0) {
      setError('请输入有效的金额');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const invoiceService = InvoiceService.getInstance();
      
      // Prepare payload directly with snake_case
      const payload = {
        payee: userInfo?.walletAddress || '', 
        payer: formData.payer, 
        amount: formData.amount,
        currency: formData.currency, 
        due_date: formData.due_date,
        invoice_ipfs_hash: formData.invoice_ipfs_hash,
        contract_ipfs_hash: formData.contract_ipfs_hash,
      };
      
      console.log('发送到后端的payload:', payload);

      // Pass the correctly formatted payload
      await invoiceService.createInvoice(payload as any); 
      setSuccess(true);
      
      // Updated reset form state with snake_case
      setFormData({
        payer: '',
        amount: 0, // 重置为0
        currency: 'CNY',
        due_date: null, // 重置为null
        invoice_ipfs_hash: '',
        contract_ipfs_hash: '',
      });

      setTimeout(() => {
        onSuccess();
        setSuccess(false);
      }, 2000);
    } catch (err) {
      console.error('Failed to create invoice:', err);
      setError(err instanceof Error ? err.message : '创建票据失败');
    } finally {
      setLoading(false);
    }
  };
  
  const handleDialogClose = () => {
    if (!loading) {
      setSuccess(false);
      onClose();
    }
  };

  return (
    <Dialog 
      open={open} 
      onClose={handleDialogClose} 
      maxWidth="md" 
      fullWidth
    >
      <DialogTitle>创建新票据</DialogTitle>
      <DialogContent>
        {success ? (
          <Alert severity="success" sx={{ mt: 2 }}>
            票据创建成功！
          </Alert>
        ) : (
          <Box sx={{ pt: 2 }}>
            {error && (
              <Alert severity="error" sx={{ mb: 2 }}>
                {error}
              </Alert>
            )}
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  label="债权人 (收款方)"
                  value={userInfo?.walletAddress || ''}
                  disabled
                  helperText="当前连接的钱包地址"
                />
              </Grid>
              {/* Debtor (Payer) */}
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  label="债务人 (付款方) 地址"
                  name="payer" 
                  value={formData.payer}
                  onChange={handleInputChange}
                  disabled={loading}
                  required
                  placeholder="0x..."
                />
              </Grid>

              <Grid item xs={12} sm={4}> 
                <TextField
                  fullWidth
                  label="金额"
                  name="amount" 
                  type="number"
                  value={formData.amount}
                  onChange={handleInputChange}
                  disabled={loading}
                  required
                  InputProps={{ inputProps: { min: 0 } }}
                />
              </Grid>

              {/* Currency */} 
              <Grid item xs={12} sm={2}> 
                <TextField
                  fullWidth
                  label="币种"
                  name="currency"
                  value={formData.currency}
                  onChange={handleInputChange}
                  disabled={loading}
                  required
                  select
                  SelectProps={{ native: true }}
                >
                  <option value="CNY">CNY</option>
                  <option value="USD">USD</option>
                  {/* Add other currencies if needed */}
                </TextField>
              </Grid>

              {/* Due Date */} 
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  label="到期日期"
                  name="due_date"
                  type="date" // Use HTML5 date input
                  value={
                    formData.due_date 
                      ? (() => {
                          // 将Unix时间戳转换回YYYY-MM-DD格式
                          const date = new Date(formData.due_date * 1000);
                          const dateStr = date.toISOString().split('T')[0];
                          console.log('显示日期时间戳:', formData.due_date);
                          console.log('转换后的日期对象:', date);
                          console.log('显示日期字符串:', dateStr);
                          return dateStr;
                        })() 
                      : ''
                  }
                  onChange={handleDateChange}
                  disabled={loading}
                  required
                  InputLabelProps={{
                    shrink: true,
                  }}
                />
              </Grid>

              {/* Invoice File Upload */} 
              <Grid item xs={12} sm={6}>
                <Button
                  component="label"
                  variant="outlined"
                  startIcon={<CloudUpload />}
                  fullWidth
                  disabled={loading || fileUploading}
                >
                  上传票据文件 {fileUploading && <CircularProgress size={20} sx={{ ml: 1}} />}
                  <input type="file" hidden onChange={(e) => handleFileUpload(e, 'invoice')} />
                </Button>
                {formData.invoice_ipfs_hash && (
                  <Typography variant="body2" sx={{ mt: 1, wordBreak: 'break-all' }}>
                    票据哈希: {formData.invoice_ipfs_hash}
                  </Typography>
                )}
              </Grid>
              {/* Contract File Upload */} 
              <Grid item xs={12} sm={6}>
                <Button
                  component="label"
                  variant="outlined"
                  startIcon={<CloudUpload />}
                  fullWidth
                  disabled={loading || fileUploading}
                >
                  上传合同文件 {fileUploading && <CircularProgress size={20} sx={{ ml: 1}} />}
                  <input type="file" hidden onChange={(e) => handleFileUpload(e, 'contract')} />
                </Button>
                {formData.contract_ipfs_hash && (
                  <Typography variant="body2" sx={{ mt: 1, wordBreak: 'break-all' }}>
                    合同哈希: {formData.contract_ipfs_hash}
                  </Typography>
                )}
              </Grid>
            </Grid>
          </Box>
        )}
      </DialogContent>
      
      {!success && (
        <DialogActions>
          <Button onClick={handleDialogClose} disabled={loading}>
            取消
          </Button>
          <Button 
            onClick={handleSubmit} 
            variant="contained" 
            color="primary"
            disabled={loading || fileUploading}
          >
            {loading ? <CircularProgress size={24} /> : '创建票据'}
          </Button>
        </DialogActions>
      )}
    </Dialog>
  );
};

export default CreateInvoiceDialog; 