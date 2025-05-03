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
  Checkbox,
  Tooltip,
} from '@mui/material';
import {
  Add as AddIcon,
  Search as SearchIcon,
  Visibility as VisibilityIcon,
  Delete as DeleteIcon,
  UploadFile as UploadIcon,
  Send as SendIcon,
} from '@mui/icons-material';
import { useAuth } from '../context/AuthContext';
import InvoiceService, { Invoice } from '../services/invoiceService';
import Layout from './Layout';
import CreateInvoiceDialog from './CreateInvoiceDialog';
import { formatDate } from '../utils/dateUtils'; // 导入日期工具函数

const InvoiceManagement: React.FC = () => {
  const { userInfo } = useAuth();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  
  // 新增状态用于管理选中的票据
  const [selectedInvoices, setSelectedInvoices] = useState<string[]>([]);
  const [processingIds, setProcessingIds] = useState<string[]>([]); // 正在处理的票据IDs
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // 获取票据列表
  useEffect(() => {
    const fetchInvoices = async () => {
      try {
        setLoading(true);
        setError(null);
        
        const invoiceService = InvoiceService.getInstance();
        const data = await invoiceService.getUserInvoices();
        console.log('Fetched Invoices:', data); // Log fetched data
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

  // 搜索过滤 (使用payee和payer字段)
  const filteredInvoices = invoices.filter((invoice: Invoice) => 
    invoice.invoice_number.toLowerCase().includes(searchTerm.toLowerCase()) ||
    (invoice.payee && invoice.payee.toLowerCase().includes(searchTerm.toLowerCase())) ||
    (invoice.payer && invoice.payer.toLowerCase().includes(searchTerm.toLowerCase()))
  );

  // 处理搜索输入
  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchTerm(e.target.value);
  };

  // 处理票据复选框选择
  const handleSelectInvoice = (invoiceId: string) => {
    setSelectedInvoices(prev => {
      if (prev.includes(invoiceId)) {
        return prev.filter(id => id !== invoiceId);
      } else {
        return [...prev, invoiceId];
      }
    });
  };

  // 检查是否可以发行选中的票据
  const canIssueSelected = (): boolean => {
    if (selectedInvoices.length === 0) return false;
    
    // 只有已上链(Verified)状态的票据可以发行
    const selectedInvoiceObjects = invoices.filter(invoice => 
      selectedInvoices.includes(invoice.id) && 
      invoice.status.toLowerCase() === 'verified'
    );
    
    if (selectedInvoiceObjects.length === 0) return false;
    
    // 检查所有选中的票据是否有相同的债权人、债务人和币种
    if (selectedInvoiceObjects.length > 1) {
      const firstInvoice = selectedInvoiceObjects[0];
      const isConsistent = selectedInvoiceObjects.every(invoice => 
        invoice.payee === firstInvoice.payee &&
        invoice.payer === firstInvoice.payer &&
        invoice.currency === firstInvoice.currency
      );
      
      if (!isConsistent) {
        // 可以添加一个提示信息
        setError('选中的票据必须具有相同的债权人、债务人和币种才能一起发行');
        return false;
      }
    }
    
    return true;
  };

  // 批量发行选中的票据
  const handleIssueSelected = async () => {
    if (!canIssueSelected() || !confirm('确定要发行选中的票据吗？')) {
      return;
    }

    try {
      setProcessingIds(selectedInvoices);
      setError(null);
      
      const invoiceService = InvoiceService.getInstance();
      await invoiceService.issueInvoices(selectedInvoices);
      
      setSuccessMessage(`成功发行 ${selectedInvoices.length} 张票据`);
      
      // 重置选择并刷新列表
      setSelectedInvoices([]);
      setRefreshTrigger(prev => prev + 1);
      
      // 3秒后清除成功消息
      setTimeout(() => {
        setSuccessMessage(null);
      }, 3000);
    } catch (err) {
      console.error('Failed to issue invoices:', err);
      setError('发行票据失败');
    } finally {
      setProcessingIds([]);
    }
  };

  // 将票据状态更新为"已上链"
  const handleVerifyInvoice = async (invoiceId: string) => {
    if (!confirm('确定要将此票据状态更新为"已上链"吗？')) {
      return;
    }

    try {
      setProcessingIds(prev => [...prev, invoiceId]);
      setError(null);
      
      const invoiceService = InvoiceService.getInstance();
      await invoiceService.verifyInvoice(invoiceId);
      
      setSuccessMessage('票据状态已更新为"已上链"');
      
      // 更新本地状态
      setInvoices(invoices.map(invoice => {
        if (invoice.id === invoiceId) {
          return { ...invoice, status: 'Verified' };
        }
        return invoice;
      }));
      
      // 3秒后清除成功消息
      setTimeout(() => {
        setSuccessMessage(null);
      }, 3000);
    } catch (err) {
      console.error('Failed to verify invoice:', err);
      setError('更新票据状态失败');
    } finally {
      setProcessingIds(prev => prev.filter(id => id !== invoiceId));
    }
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
      setProcessingIds(prev => [...prev, id]);
      const invoiceService = InvoiceService.getInstance();
      await invoiceService.deleteInvoice(id);
      
      // 从列表中移除该票据
      setInvoices(invoices.filter((invoice: Invoice) => invoice.id !== id));
      
      // 如果该票据在选中状态，也从选中列表中移除
      if (selectedInvoices.includes(id)) {
        setSelectedInvoices(prev => prev.filter(invoiceId => invoiceId !== id));
      }
    } catch (err) {
      console.error('Failed to delete invoice:', err);
      setError('删除票据失败');
    } finally {
      setProcessingIds(prev => prev.filter(pid => pid !== id));
    }
  };

  // 获取票据状态显示 (与后端状态中文描述保持一致)
  const getStatusChip = (status?: string) => {
    if (!status) return <Chip label="未知" size="small" />;
    
    const lowerCaseStatus = status.toLowerCase();
    
    switch (lowerCaseStatus) {
      case 'pending':
        return <Chip label="未上链" color="warning" size="small" />;
      case 'verified':
        return <Chip label="已上链" color="success" size="small" />;
      case 'packaged':
        return <Chip label="已包含在发票批次中" color="info" size="small" />;
      case 'repaid':
        return <Chip label="已清算" color="success" size="small" />;
      case 'overdue':
        return <Chip label="已逾期" color="error" size="small" />;
      case 'defaulted':
        return <Chip label="已违约" color="error" size="small" />;
      case 'onsale':
        return <Chip label="在售" color="primary" size="small" />;
      case 'soldout':
        return <Chip label="已售出" color="secondary" size="small" />;
      default:
        // Capitalize first letter for display
        const displayStatus = status.charAt(0).toUpperCase() + status.slice(1);
        return <Chip label={displayStatus} size="small" />;
    }
  };

  // Helper to shorten addresses
  const shortenAddress = (address?: string) => {
    if (!address) return 'N/A';
    if (address.length <= 14) return address; // Avoid shortening already short strings
    return `${address.substring(0, 8)}...${address.substring(address.length - 6)}`;
  }

  // 生成IPFS预览链接
  const getIpfsLink = (ipfsHash?: string) => {
    if (!ipfsHash) return 'N/A';
    return (
      <a 
        href={`https://ipfs.io/ipfs/${ipfsHash}`} 
        target="_blank" 
        rel="noopener noreferrer" 
        style={{ textDecoration: 'none' }}
      >
        {shortenAddress(ipfsHash)}
      </a>
    );
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
              placeholder="搜索票据编号或债权人/债务人地址"
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
            <Box>
              {/* 发行按钮，只在有选中票据且符合条件时启用 */}
              <Button
                variant="contained"
                color="secondary"
                startIcon={<SendIcon />}
                onClick={handleIssueSelected}
                disabled={!canIssueSelected() || processingIds.length > 0}
                sx={{ mr: 2 }}
              >
                发行到市场
              </Button>
              
              <Button
                variant="contained"
                color="primary"
                startIcon={<AddIcon />}
                onClick={handleOpenCreateDialog}
              >
                创建票据
              </Button>
            </Box>
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
        
        {successMessage && (
          <Alert severity="success" sx={{ mb: 3 }}>
            {successMessage}
          </Alert>
        )}

        {loading && !processingIds.length ? (
          <Box sx={{ display: 'flex', justifyContent: 'center', my: 5 }}>
            <CircularProgress />
          </Box>
        ) : filteredInvoices.length > 0 ? (
          <TableContainer component={Paper}>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell padding="checkbox">
                    {/* 可以在这里添加全选功能 */}
                  </TableCell>
                  <TableCell>票据编号</TableCell>
                  <TableCell>债权人地址</TableCell>
                  <TableCell>债务人地址</TableCell>
                  <TableCell>金额</TableCell>
                  <TableCell>币种</TableCell>
                  <TableCell>到期日</TableCell>
                  <TableCell>票据地址</TableCell>
                  <TableCell>合同地址</TableCell>
                  <TableCell>状态</TableCell>
                  <TableCell>操作</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {filteredInvoices.map((invoice: Invoice) => (
                  <TableRow key={invoice.id}>
                    <TableCell padding="checkbox">
                      <Checkbox
                        checked={selectedInvoices.includes(invoice.id)}
                        onChange={() => handleSelectInvoice(invoice.id)}
                        disabled={processingIds.includes(invoice.id)}
                      />
                    </TableCell>
                    <TableCell>{invoice.invoice_number}</TableCell>
                    {/* 显示payee字段作为债权人地址 */}
                    <TableCell title={invoice.payee || ''}>
                      {shortenAddress(invoice.payee)}
                    </TableCell>
                    {/* 显示payer字段作为债务人地址 */}
                    <TableCell title={invoice.payer || ''}>
                      {shortenAddress(invoice.payer)}
                    </TableCell>
                    <TableCell>{invoice.amount}</TableCell>
                    <TableCell>{invoice.currency || 'CNY'}</TableCell>
                    <TableCell>{formatDate(invoice.due_date)}</TableCell>
                    {/* 显示票据IPFS地址 */}
                    <TableCell>
                      {getIpfsLink(invoice.invoice_ipfs_hash)}
                    </TableCell>
                    {/* 显示合同IPFS地址 */}
                    <TableCell>
                      {getIpfsLink(invoice.contract_ipfs_hash)}
                    </TableCell>
                    <TableCell>{getStatusChip(invoice.status)}</TableCell>
                    <TableCell>
                      {/* 未上链状态才显示上链按钮 */}
                      {invoice.status.toLowerCase() === 'pending' && (
                        <Tooltip title="上链">
                          <IconButton
                            size="small"
                            onClick={() => handleVerifyInvoice(invoice.id)}
                            disabled={processingIds.includes(invoice.id)}
                            color="primary"
                          >
                            <UploadIcon fontSize="small" />
                          </IconButton>
                        </Tooltip>
                      )}
                      
                      <Tooltip title="查看详情">
                        <IconButton 
                          size="small" 
                          onClick={() => alert(`查看票据详情: ${invoice.id} 功能待实现`)}
                          disabled={processingIds.includes(invoice.id)}
                        >
                          <VisibilityIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                      
                      {/* 只允许删除未上链或者已上链状态的票据 */}
                      {userInfo?.isEnterpriseBound && 
                       ['pending', 'verified'].includes(invoice.status.toLowerCase()) && (
                        <Tooltip title="删除">
                          <IconButton 
                            size="small" 
                            color="error"
                            onClick={() => handleDeleteInvoice(invoice.id)}
                            disabled={processingIds.includes(invoice.id)}
                          >
                            <DeleteIcon fontSize="small" />
                          </IconButton>
                        </Tooltip>
                      )}
                      
                      {/* 正在处理中显示加载图标 */}
                      {processingIds.includes(invoice.id) && (
                        <CircularProgress size={20} sx={{ ml: 1 }} />
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        ) : (
          <Typography variant="body1" textAlign="center" sx={{ my: 5 }}>
            {searchTerm ? '未找到匹配的票据记录' : '没有票据记录'}
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