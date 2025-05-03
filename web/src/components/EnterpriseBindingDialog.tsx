import React, { useState } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  Typography,
  CircularProgress,
  Box,
  Alert
} from '@mui/material';
import { BusinessCenter } from '@mui/icons-material';
import { useAuth } from '../context/AuthContext';

interface EnterpriseBindingDialogProps {
  open: boolean;
  onClose: () => void;
}

const EnterpriseBindingDialog: React.FC<EnterpriseBindingDialogProps> = ({ open, onClose }) => {
  const [enterpriseAddress, setEnterpriseAddress] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const { bindEnterprise, isBindingEnterprise } = useAuth();

  const handleAddressChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setEnterpriseAddress(e.target.value);
    setError(null);
  };

  const handleBindEnterprise = async () => {
    // Basic validation
    if (!enterpriseAddress) {
      setError('企业钱包地址不能为空');
      return;
    }

    if (!enterpriseAddress.startsWith('0x') || enterpriseAddress.length !== 42) {
      setError('无效的企业钱包地址格式');
      return;
    }

    try {
      const result = await bindEnterprise(enterpriseAddress);
      if (result) {
        setSuccess(true);
        // Close dialog after a short delay to show success message
        setTimeout(() => {
          onClose();
          window.location.reload(); // Reload to update navigation
        }, 1500);
      } else {
        setError('绑定企业失败，请稍后重试');
      }
    } catch (err) {
      setError('绑定企业过程中发生错误');
      console.error('Error binding enterprise:', err);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <BusinessCenter color="primary" />
        企业认证绑定
      </DialogTitle>
      
      <DialogContent>
        {success ? (
          <Alert severity="success" sx={{ my: 2 }}>
            企业绑定成功！您现在可以访问企业功能。
          </Alert>
        ) : (
          <>
            <Typography variant="body1" sx={{ mb: 2 }}>
              绑定企业钱包地址可以使您获得企业功能权限，包括发行债权、管理票据等。
            </Typography>
            
            <TextField
              label="企业钱包地址"
              placeholder="0x..."
              fullWidth
              value={enterpriseAddress}
              onChange={handleAddressChange}
              error={!!error}
              helperText={error}
              disabled={isBindingEnterprise}
              sx={{ mb: 2 }}
            />
            
            {isBindingEnterprise && (
              <Box sx={{ display: 'flex', justifyContent: 'center', my: 2 }}>
                <CircularProgress size={24} />
              </Box>
            )}
          </>
        )}
      </DialogContent>
      
      <DialogActions>
        <Button onClick={onClose} disabled={isBindingEnterprise}>
          取消
        </Button>
        <Button 
          onClick={handleBindEnterprise} 
          variant="contained" 
          color="primary"
          disabled={isBindingEnterprise || success || !enterpriseAddress}
        >
          {isBindingEnterprise ? '绑定中...' : '绑定企业'}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default EnterpriseBindingDialog; 