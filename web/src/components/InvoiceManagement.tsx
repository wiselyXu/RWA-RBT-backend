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
  Chip,
  IconButton,
  TextField,
  InputAdornment,
  Grid,
  CircularProgress,
  Alert,
} from '@mui/material';
import {
  Add as AddIcon,
  Search as SearchIcon,
  Visibility as VisibilityIcon,
  Delete as DeleteIcon,
} from '@mui/icons-material';
import { useAuth } from '../context/AuthContext';
import InvoiceService, { Invoice } from '../services/invoiceService';
import Layout from './Layout';
import CreateInvoiceDialog from './CreateInvoiceDialog';

const InvoiceManagement: React.FC = () => {
  const { userInfo } = useAuth();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  // 获取票据列表
  useEffect(() => {
    const fetchInvoices = async () => {
      try {
        setLoading(true);
        setError(null);
        
        const invoiceService = InvoiceService.getInstance();
        const data = await invoiceService.getUserInvoices();
        setInvoices(data);
      } catch (err) {
        console.error('Failed to fetch invoices:', err);
        setError('获取票据列表失败');
      } finally {
        setLoading(false);
      }
    };

    fetchInvoices();
  }, [refreshTrigger]); // 当refreshTrigger变化时，重新获取数据

  // 搜索过滤
  const filteredInvoices = invoices.filter((invoice: Invoice) => 
    invoice.invoice_number.toLowerCase().includes(searchTerm.toLowerCase()) ||
    (invoice.payer && invoice.payer.toLowerCase().includes(searchTerm.toLowerCase()))
  );

  // 处理搜索输入
  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchTerm(e.target.value);
  };

  // 打开创建票据对话框
  const handleOpenCreateDialog = () => {
    setCreateDialogOpen(true);
  };

  // 关闭创建票据对话框
  const handleCloseCreateDialog = () => {
    setCreateDialogOpen(false);
  };

  // 票据创建成功后的回调
  const handleCreateSuccess = () => {
    setCreateDialogOpen(false);
    // 触发列表刷新
    setRefreshTrigger((prev: number) => prev + 1);
  };

  // 删除票据
  const handleDeleteInvoice = async (id: string) => {
    if (!confirm('确定要删除此票据吗？')) {
      return;
    }

    try {
      setLoading(true);
      const invoiceService = InvoiceService.getInstance();
      await invoiceService.deleteInvoice(id);
      
      // 从列表中移除该票据
      setInvoices(invoices.filter((invoice: Invoice) => invoice.id !== id));
    } catch (err) {
      console.error('Failed to delete invoice:', err);
      alert('删除票据失败');
    } finally {
      setLoading(false);
    }
  };

  // 获取票据状态显示
  const getStatusChip = (status: string) => {
    switch (status.toLowerCase()) {
      case 'pending':
        return <Chip label="待处理" color="warning" size="small" />;
      case 'verified':
        return <Chip label="已验证" color="success" size="small" />;
      case 'repaid':
        return <Chip label="已偿还" color="info" size="small" />;
      default:
        return <Chip label={status} size="small" />;
    }
  };

  return (
    <Layout>
      <Box sx={{ p: 3 }}>
        <Typography variant="h5" gutterBottom>
          票据管理
        </Typography>

        {/* 只对已绑定企业的用户显示创建按钮 */}
        {userInfo?.isEnterpriseBound ? (
          <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
            <TextField
              placeholder="搜索票据编号或债务方"
              variant="outlined"
              size="small"
              value={searchTerm}
              onChange={handleSearchChange}
              InputProps={{
                startAdornment: (
                  <InputAdornment position="start">
                    <SearchIcon />
                  </InputAdornment>
                ),
              }}
              sx={{ width: 300 }}
            />
            <Button
              variant="contained"
              color="primary"
              startIcon={<AddIcon />}
              onClick={handleOpenCreateDialog}
            >
              创建票据
            </Button>
          </Box>
        ) : (
          <Alert severity="info" sx={{ mb: 3 }}>
            只有已认证的企业用户才能创建和管理票据。请先完成企业认证。
          </Alert>
        )}

        {error && (
          <Alert severity="error" sx={{ mb: 3 }}>
            {error}
          </Alert>
        )}

        {loading ? (
          <Box sx={{ display: 'flex', justifyContent: 'center', my: 5 }}>
            <CircularProgress />
          </Box>
        ) : filteredInvoices.length > 0 ? (
          <TableContainer component={Paper}>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>票据编号</TableCell>
                  <TableCell>债权人</TableCell>
                  <TableCell>债务方</TableCell>
                  <TableCell>金额</TableCell>
                  <TableCell>到期日</TableCell>
                  <TableCell>状态</TableCell>
                  <TableCell>操作</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {filteredInvoices.map((invoice: Invoice) => (
                  <TableRow key={invoice.id}>
                    <TableCell>{invoice.invoice_number}</TableCell>
                    <TableCell title={invoice.payee || ''}>
                      {invoice.payee ? `${invoice.payee.substring(0, 8)}...${invoice.payee.substring(invoice.payee.length - 6)}` : ''}
                    </TableCell>
                    <TableCell title={invoice.payer || ''}>
                      {invoice.payer ? `${invoice.payer.substring(0, 8)}...${invoice.payer.substring(invoice.payer.length - 6)}` : ''}
                    </TableCell>
                    <TableCell>{invoice.amount}</TableCell>
                    <TableCell>{new Date(invoice.due_date).toLocaleDateString()}</TableCell>
                    <TableCell>{getStatusChip(invoice.status)}</TableCell>
                    <TableCell>
                      <IconButton 
                        size="small" 
                        title="查看详情"
                        onClick={() => alert('查看详情功能待实现')}
                      >
                        <VisibilityIcon fontSize="small" />
                      </IconButton>
                      {userInfo?.isEnterpriseBound && (
                        <IconButton 
                          size="small" 
                          title="删除"
                          onClick={() => handleDeleteInvoice(invoice.id)}
                        >
                          <DeleteIcon fontSize="small" />
                        </IconButton>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        ) : (
          <Typography variant="body1" textAlign="center" sx={{ my: 5 }}>
            没有找到票据记录
          </Typography>
        )}
      </Box>

      {/* 创建票据对话框 */}
      <CreateInvoiceDialog
        open={createDialogOpen}
        onClose={handleCloseCreateDialog}
        onSuccess={handleCreateSuccess}
      />
    </Layout>
  );
};

export default InvoiceManagement; 