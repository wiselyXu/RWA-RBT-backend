import React, { useState } from 'react';
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
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDateFns } from '@mui/x-date-pickers/AdapterDateFns';
import { CloudUpload } from '@mui/icons-material';
import { useAuth } from '../context/AuthContext';
import InvoiceService, { CreateInvoiceParams } from '../services/invoiceService';

interface CreateInvoiceDialogProps {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

interface FormData {
  invoice_number: string;
  payer: string;
  amount: string;
  due_date: Date | null;
  ipfs_hash: string;
  contract_hash: string;
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
  
  // 表单数据
  const [formData, setFormData] = useState<FormData>({
    invoice_number: '',
    payer: '',
    amount: '',
    due_date: null,
    ipfs_hash: '',
    contract_hash: '',
  });

  // 更新表单数据
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData((prev: FormData) => ({
      ...prev,
      [name]: value,
    }));
  };

  // 日期选择处理
  const handleDateChange = (date: Date | null) => {
    setFormData((prev: FormData) => ({
      ...prev,
      due_date: date,
    }));
  };

  // 文件上传处理 (实际项目中可能需要实现文件上传到IPFS或其他存储)
  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>, type: 'invoice' | 'contract') => {
    if (!e.target.files || e.target.files.length === 0) return;
    
    try {
      setFileUploading(true);
      // 模拟上传文件过程
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      // 这里应该实现实际的文件上传逻辑，然后获取哈希值
      const mockHash = `ipfs_hash_${Math.random().toString(36).substring(2, 10)}`;
      
      if (type === 'invoice') {
        setFormData((prev: FormData) => ({
          ...prev,
          ipfs_hash: mockHash,
        }));
      } else {
        setFormData((prev: FormData) => ({
          ...prev,
          contract_hash: mockHash,
        }));
      }
    } catch (err) {
      console.error(`Failed to upload ${type} file:`, err);
      setError(`上传${type === 'invoice' ? '票据' : '合同'}文件失败`);
    } finally {
      setFileUploading(false);
    }
  };

  // 提交表单
  const handleSubmit = async () => {
    // 表单验证
    if (!formData.invoice_number || !formData.payer || !formData.amount || !formData.due_date || !formData.ipfs_hash || !formData.contract_hash) {
      setError('请填写所有必填字段');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const invoiceService = InvoiceService.getInstance();
      
      // 准备创建票据的参数
      const invoiceData: CreateInvoiceParams = {
        invoice_number: formData.invoice_number,
        payee: userInfo?.walletAddress || '',
        payer: formData.payer,
        amount: formData.amount,
        ipfs_hash: formData.ipfs_hash,
        contract_hash: formData.contract_hash,
        timestamp: Math.floor(Date.now() / 1000).toString(),
        due_date: Math.floor(formData.due_date!.getTime() / 1000).toString(),
        token_batch: '', // 初始创建时不需要填写
        is_cleared: false,
        is_valid: true,
      };

      await invoiceService.createInvoice(invoiceData);
      setSuccess(true);
      
      // 重置表单
      setFormData({
        invoice_number: '',
        payer: '',
        amount: '',
        due_date: null,
        ipfs_hash: '',
        contract_hash: '',
      });

      // 3秒后关闭对话框
      setTimeout(() => {
        onSuccess();
      }, 2000);
    } catch (err) {
      console.error('Failed to create invoice:', err);
      setError(err instanceof Error ? err.message : '创建票据失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog 
      open={open} 
      onClose={loading ? undefined : onClose} 
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
                  label="票据编号"
                  name="invoice_number"
                  value={formData.invoice_number}
                  onChange={handleInputChange}
                  disabled={loading}
                  required
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  label="债权人账户"
                  value={userInfo?.walletAddress || ''}
                  disabled
                  helperText="使用当前连接的钱包地址"
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  label="债务方地址"
                  name="payer"
                  value={formData.payer}
                  onChange={handleInputChange}
                  disabled={loading}
                  required
                  placeholder="0x..."
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  label="金额"
                  name="amount"
                  type="number"
                  value={formData.amount}
                  onChange={handleInputChange}
                  disabled={loading}
                  required
                  InputProps={{
                    endAdornment: <InputAdornment position="end">元</InputAdornment>,
                  }}
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <LocalizationProvider dateAdapter={AdapterDateFns}>
                  <DatePicker 
                    label="到期日期"
                    value={formData.due_date}
                    onChange={handleDateChange}
                    disabled={loading}
                    slotProps={{
                      textField: {
                        fullWidth: true,
                        required: true
                      }
                    }}
                  />
                </LocalizationProvider>
              </Grid>
              
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  label="企业名称"
                  value={userInfo?.enterpriseName || '未绑定企业'}
                  disabled
                />
              </Grid>
              
              <Grid item xs={12}>
                <Button
                  variant="outlined"
                  component="label"
                  startIcon={<CloudUpload />}
                  fullWidth
                  disabled={loading || fileUploading}
                >
                  {fileUploading ? '上传中...' : '上传票据图片'}
                  <input
                    type="file"
                    hidden
                    accept="image/*,.pdf"
                    onChange={(e) => handleFileUpload(e, 'invoice')}
                  />
                </Button>
                {formData.ipfs_hash && (
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                    已上传票据文件: {formData.ipfs_hash}
                  </Typography>
                )}
              </Grid>
              
              <Grid item xs={12}>
                <Button
                  variant="outlined"
                  component="label"
                  startIcon={<CloudUpload />}
                  fullWidth
                  disabled={loading || fileUploading}
                >
                  {fileUploading ? '上传中...' : '上传合同文件'}
                  <input
                    type="file"
                    hidden
                    accept="image/*,.pdf"
                    onChange={(e) => handleFileUpload(e, 'contract')}
                  />
                </Button>
                {formData.contract_hash && (
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                    已上传合同文件: {formData.contract_hash}
                  </Typography>
                )}
              </Grid>
            </Grid>
          </Box>
        )}
      </DialogContent>
      
      {!success && (
        <DialogActions>
          <Button onClick={onClose} disabled={loading}>
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