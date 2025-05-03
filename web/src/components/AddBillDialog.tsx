import React from 'react';
import {
  Box,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Grid,
  TextField,
  FormControlLabel,
  Switch,
  Typography,
  InputAdornment,

} from '@mui/material';
import { CloudUpload } from '@mui/icons-material';

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

interface AddBillDialogProps {
  open: boolean;
  onClose: () => void;
  onSubmit: () => void;
  currentBill: Bill;
  isEditing: boolean;
  handleInputChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  handleFileUpload: (e: React.ChangeEvent<HTMLInputElement>, type: 'bill' | 'contract') => void;
  initialBill: Bill;
}

const AddBillDialog: React.FC<AddBillDialogProps> = ({
  open,
  onClose,
  onSubmit,
  currentBill,
  isEditing,
  handleInputChange,
  handleFileUpload,
  initialBill,
}) => {
  return (
    <Dialog 
      open={open} 
      onClose={onClose} 
      maxWidth="sm" 
      fullWidth
      PaperProps={{
        sx: {
          maxWidth: '500px'
        }
      }}
    >
      <DialogTitle>{isEditing ? '编辑票据' : '新增票据'}</DialogTitle>
      <DialogContent>
        <Box sx={{ pt: 2 }}>
          <Grid container spacing={2}>
            {isEditing ? (
              <>
                <Grid item xs={12} sm={6}>
                  <TextField
                    fullWidth
                    label="票据编号"
                    name="id"
                    value={currentBill.id}
                    onChange={handleInputChange}
                    required
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <TextField
                    fullWidth
                    label="债权人账户"
                    name="creditorAccount"
                    value={currentBill.creditorAccount}
                    disabled
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <TextField
                    fullWidth
                    label="债务方"
                    name="debtor"
                    value={currentBill.debtor}
                    onChange={handleInputChange}
                    required
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <TextField
                    fullWidth
                    label="金额"
                    name="amount"
                    type="number"
                    value={currentBill.amount}
                    onChange={handleInputChange}
                    required
                    InputProps={{
                      endAdornment: <InputAdornment position="end">元</InputAdornment>,
                    }}
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <TextField
                    fullWidth
                    label="Token批次编码"
                    name="tokenBatchCode"
                    value={currentBill.tokenBatchCode}
                    disabled
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <TextField
                    fullWidth
                    label="创建钱包"
                    name="createdWallet"
                    value={currentBill.createdWallet}
                    disabled
                  />
                </Grid>
                <Grid item xs={12}>
                  <Button
                    variant="outlined"
                    component="label"
                    startIcon={<CloudUpload />}
                    fullWidth
                    disabled
                  >
                    上传票据图片
                    <input
                      type="file"
                      hidden
                      accept="image/*"
                      onChange={(e) => handleFileUpload(e, 'bill')}
                    />
                  </Button>
                  {currentBill.billImageUrl && (
                    <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                      已上传票据图片
                    </Typography>
                  )}
                </Grid>
                <Grid item xs={12}>
                  <Button
                    variant="outlined"
                    component="label"
                    startIcon={<CloudUpload />}
                    fullWidth
                    disabled
                  >
                    上传合同图片
                    <input
                      type="file"
                      hidden
                      accept="image/*,.pdf,.zip"
                      onChange={(e) => handleFileUpload(e, 'contract')}
                    />
                  </Button>
                  {currentBill.contractImageUrl && (
                    <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                      已上传合同文件
                    </Typography>
                  )}
                </Grid>
                <Grid item xs={12} sm={6}>
                  <FormControlLabel
                    control={
                      <Switch
                        checked={currentBill.isCleared}
                        onChange={handleInputChange}
                        name="isCleared"
                        disabled={true}
                      />
                    }
                    label="是否已清算"
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <FormControlLabel
                    control={
                      <Switch
                        checked={currentBill.isOnChain}
                        onChange={handleInputChange}
                        name="isOnChain"
                        disabled={true}
                      />
                    }
                    label="是否已上链"
                  />
                </Grid>
              </>
            ) : (
              <>
                <Grid item xs={12}>
                  <Grid container spacing={2}>
                    <Grid item xs={6}>
                      <TextField
                        fullWidth
                        label="票据编号"
                        name="id"
                        value={currentBill.id}
                        onChange={handleInputChange}
                        required
                      />
                    </Grid>
                    <Grid item xs={6}>
                      <TextField
                        fullWidth
                        label="金额"
                        name="amount"
                        type="number"
                        value={currentBill.amount}
                        onChange={handleInputChange}
                        required
                        InputProps={{
                          endAdornment: <InputAdornment position="end">元</InputAdornment>,
                        }}
                      />
                    </Grid>
                  </Grid>
                </Grid>
                <Grid item xs={12}>
                  <TextField
                    fullWidth
                    label="债务方"
                    name="debtor"
                    value={currentBill.debtor}
                    onChange={handleInputChange}
                    required
                  />
                </Grid>
                <Grid item xs={12}>
                  <Grid container spacing={2}>
                    <Grid item xs={6}>
                      <Button
                        variant="outlined"
                        component="label"
                        startIcon={<CloudUpload />}
                        fullWidth
                      >
                        上传票据图片
                        <input
                          type="file"
                          hidden
                          accept="image/*"
                          onChange={(e) => handleFileUpload(e, 'bill')}
                        />
                      </Button>
                      {currentBill.billImageUrl && (
                        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                          已上传票据图片
                        </Typography>
                      )}
                    </Grid>
                    <Grid item xs={6}>
                      <Button
                        variant="outlined"
                        component="label"
                        startIcon={<CloudUpload />}
                        fullWidth
                      >
                        上传合同图片
                        <input
                          type="file"
                          hidden
                          accept="image/*,.pdf,.zip"
                          onChange={(e) => handleFileUpload(e, 'contract')}
                        />
                      </Button>
                      {currentBill.contractImageUrl && (
                        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                          已上传合同文件
                        </Typography>
                      )}
                    </Grid>
                  </Grid>
                </Grid>
              </>
            )}
          </Grid>
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>取消</Button>
        <Button onClick={onSubmit} variant="contained" color="primary">
          确定
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default AddBillDialog;
