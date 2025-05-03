import React from 'react';
import { 
  Dialog, 
  DialogTitle, 
  DialogContent, 
  DialogActions, 
  Box, 
  Grid, 
  Typography, 
  Chip, 
  Card, 
  CardMedia, 
  Button 
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
}

interface Props {
  detailOpen: boolean;
  detailBill: Bill | null;
  handleCloseDetail: () => void;
}

const BillDetailDialog: React.FC<Props> = ({ detailOpen, detailBill, handleCloseDetail }) => {
  return (
    <Dialog
      open={detailOpen}
      onClose={handleCloseDetail}
      maxWidth="md"
      fullWidth
    >
      <DialogTitle>票据详情</DialogTitle>
      <DialogContent>
        {detailBill && (
          <Box sx={{ pt: 2 }}>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6}>
                <Typography variant="subtitle2" color="text.secondary">票据编号</Typography>
                <Typography variant="body1">{detailBill.id}</Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="subtitle2" color="text.secondary">债权人账户</Typography>
                <Typography variant="body1">{detailBill.creditorAccount}</Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="subtitle2" color="text.secondary">债务方</Typography>
                <Typography variant="body1">{detailBill.debtor}</Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="subtitle2" color="text.secondary">金额</Typography>
                <Typography variant="body1">{detailBill.amount.toLocaleString()} 元</Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="subtitle2" color="text.secondary">Token批次号</Typography>
                <Typography variant="body1">{detailBill.tokenBatchCode}</Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="subtitle2" color="text.secondary">创建钱包</Typography>
                <Typography variant="body1">{detailBill.createdWallet}</Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="subtitle2" color="text.secondary">创建时间</Typography>
                <Typography variant="body1">{detailBill.createdAt.toLocaleString()}</Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="subtitle2" color="text.secondary">修改时间</Typography>
                <Typography variant="body1">{detailBill.updatedAt.toLocaleString()}</Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="subtitle2" color="text.secondary">修改钱包</Typography>
                <Typography variant="body1">{detailBill.updatedWallet}</Typography>
              </Grid>
              <Grid item xs={12}>
                <Typography variant="subtitle2" color="text.secondary">状态</Typography>
                <Box sx={{ display: 'flex', gap: 1, mt: 1 }}>
                  <Chip
                    label={detailBill.isOnChain ? '已上链' : '未上链'}
                    color={detailBill.isOnChain ? 'success' : 'default'}
                  />
                  <Chip
                    label={detailBill.isCleared ? '已清算' : '未清算'}
                    color={detailBill.isCleared ? 'primary' : 'default'}
                  />
                </Box>
              </Grid>
              <Grid item xs={12}>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  票据图片
                </Typography>
                <Card sx={{ maxWidth: '100%', mb: 2 }}>
                  <CardMedia
                    component="img"
                    image={detailBill.billImageUrl}
                    alt="票据图片"
                    sx={{
                      height: 300,
                      objectFit: 'contain',
                      bgcolor: '#f5f5f5'
                    }}
                  />
                </Card>
              </Grid>
              <Grid item xs={12}>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  合同图片
                </Typography>
                <Card sx={{ maxWidth: '100%' }}>
                  <CardMedia
                    component="img"
                    image={detailBill.contractImageUrl}
                    alt="合同图片"
                    sx={{
                      height: 300,
                      objectFit: 'contain',
                      bgcolor: '#f5f5f5'
                    }}
                  />
                </Card>
              </Grid>
            </Grid>
          </Box>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={handleCloseDetail}>关闭</Button>
      </DialogActions>
    </Dialog>
  );
};

export default BillDetailDialog;
