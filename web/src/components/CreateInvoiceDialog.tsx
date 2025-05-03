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

// Assuming ethers is installed or will be added
// import { ethers } from 'ethers';

interface CreateInvoiceDialogProps {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

// FormData without invoiceNumber
interface FormData {
  payer: string; 
  amount: string;
  currency: string; // Added currency
  dueDate: Date | null; // Keep Date object for picker
  ipfsHash: string; 
  contractHash: string; 
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
  
  // Form state without invoiceNumber
  const [formData, setFormData] = useState<FormData>({
    payer: '',
    amount: '',
    currency: 'CNY', // Default currency
    dueDate: null,
    ipfsHash: '',
    contractHash: '',
  });

  // Update form data
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData((prev: FormData) => ({
      ...prev,
      [name]: value,
    }));
  };

  // Handle date change
  const handleDateChange = (date: Date | null) => {
    setFormData((prev: FormData) => ({
      ...prev,
      dueDate: date,
    }));
  };

  // Handle file upload
  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>, type: 'invoice' | 'contract') => {
    if (!e.target.files || e.target.files.length === 0) return;
    
    try {
      setFileUploading(true);
      await new Promise(resolve => setTimeout(resolve, 1500)); 
      const mockHash = `mock_${type}_hash_${Math.random().toString(36).substring(2, 10)}`;
      
      // Use camelCase for state update
      if (type === 'invoice') {
        setFormData((prev: FormData) => ({ ...prev, ipfsHash: mockHash }));
      } else {
        setFormData((prev: FormData) => ({ ...prev, contractHash: mockHash }));
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
    // Validation without invoiceNumber
    if (/*!formData.invoiceNumber || */ !formData.payer || !formData.amount || !formData.currency || !formData.dueDate || !formData.ipfsHash || !formData.contractHash) {
      setError('请填写所有必填字段');
      return;
    }
    
    // Validate amount
    const amountValue = parseFloat(formData.amount);
    if (isNaN(amountValue) || amountValue <= 0) {
      setError('请输入有效的金额');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const invoiceService = InvoiceService.getInstance();

      // --- Amount Conversion (Placeholder) ---
      // TODO: Convert amount to U256 string format using ethers.js or similar
      // Example (requires ethers): 
      // const amountU256String = ethers.utils.parseUnits(formData.amount, 18).toString(); 
      const amountToSend = formData.amount; // Using raw string for now

      // --- Due Date Conversion to ISO String ---
      const dueDateISOString = formData.dueDate!.toISOString(); // Use ISO format
      
      // Prepare payload matching CreateInvoiceParams (camelCase)
      const invoiceData: CreateInvoiceParams = {
        payee: userInfo?.walletAddress || '', 
        payer: formData.payer, 
        amount: amountToSend, 
        currency: formData.currency, 
        dueDate: dueDateISOString, // Send ISO string
        ipfsHash: formData.ipfsHash,
        contractHash: formData.contractHash,
      };

      await invoiceService.createInvoice(invoiceData);
      setSuccess(true);
      
      // Reset form state without invoiceNumber
      setFormData({
        payer: '',
        amount: '',
        currency: 'CNY',
        dueDate: null,
        ipfsHash: '',
        contractHash: '',
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
              {/* Invoice Number Field Removed */}
              {/* <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  label="票据编号"
                  name="invoiceNumber" 
                  value={formData.invoiceNumber}
                  onChange={handleInputChange}
                  disabled={loading}
                  required
                />
              </Grid> */}
              {/* Creditor (Payee) */}
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
              {/* Amount */            
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
              {/* Removed comment from here */}
              <Grid item xs={12} sm={2}> 
                 {/* Currency Field Re-added */}
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
                    {/* Add other currencies as needed */}
                  </TextField>
              </Grid>
              {/* Due Date */}
              <Grid item xs={12} sm={6}>
                <LocalizationProvider dateAdapter={AdapterDateFns}>
                  <>
                    <DatePicker 
                      label="到期日期"
                      value={formData.dueDate} // Use camelCase state variable
                      onChange={handleDateChange}
                      disabled={loading}
                      slotProps={{
                        textField: {
                          fullWidth: true,
                          required: true,
                          name: 'dueDate' // Bind to camelCase state
                        }
                      }}
                    />
                  </>
                </LocalizationProvider>
              </Grid>
              {/* Annual Interest Rate field removed */}
              {/* Enterprise Name - Display only */}
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  label="所属企业"
                  value={userInfo?.enterpriseName || '未绑定企业'}
                  disabled
                />
              </Grid>
              
              {/* IPFS Hash Upload */}
              <Grid item xs={12}>
                <Button
                  variant="outlined"
                  component="label"
                  startIcon={<CloudUpload />}
                  fullWidth
                  disabled={loading || fileUploading}
                >
                  {fileUploading ? '上传中...' : '上传票据文件 (IPFS)'}
                  <input
                    type="file"
                    hidden
                    accept="image/*,.pdf"
                    onChange={(e) => handleFileUpload(e, 'invoice')}
                  />
                </Button>
                {formData.ipfsHash && (
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                    票据 IPFS Hash: {formData.ipfsHash}
                  </Typography>
                )}
              </Grid>
              
              {/* Contract Hash Upload */}
              <Grid item xs={12}>
                <Button
                  variant="outlined"
                  component="label"
                  startIcon={<CloudUpload />}
                  fullWidth
                  disabled={loading || fileUploading}
                >
                  {fileUploading ? '上传中...' : '上传合同文件 (IPFS)'}
                  <input
                    type="file"
                    hidden
                    accept="image/*,.pdf,.doc,.docx"
                    onChange={(e) => handleFileUpload(e, 'contract')}
                  />
                </Button>
                {formData.contractHash && (
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                    合同 IPFS Hash: {formData.contractHash}
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
            // Update disabled check without invoiceNumber
            disabled={loading || fileUploading || /* !formData.invoiceNumber || */ !formData.payer || !formData.amount || !formData.currency || !formData.dueDate || !formData.ipfsHash || !formData.contractHash}
          >
            {loading ? <CircularProgress size={24} /> : '创建票据'}
          </Button>
        </DialogActions>
      )}
    </Dialog>
  );
};

export default CreateInvoiceDialog; 