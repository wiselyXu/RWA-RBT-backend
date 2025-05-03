import React, { useState, useMemo, useEffect } from 'react';
import {
  Box,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  IconButton,
  Typography,
  Grid,
  FormControlLabel,
  Switch,
  InputAdornment,
  TablePagination,
  Chip,
  Checkbox,
  Link,
  Card,
  CardMedia,
  Toolbar,
  InputBase,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Alert,
  Tooltip,
} from '@mui/material';
import { Edit as EditIcon, Delete as DeleteIcon, Add as AddIcon, CloudUpload as CloudUploadIcon, Search as SearchIcon, Visibility as VisibilityIcon, CheckCircle as CheckCircleIcon, Cancel as CancelIcon, Link as LinkIcon } from '@mui/icons-material';
import {
  useReactTable,
  getCoreRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  flexRender,
  createColumnHelper,
  SortingState,
  OnChangeFn,
} from '@tanstack/react-table';
import WalletService from '../services/walletService';
import Layout from './Layout';
import AddBillDialog from './AddBillDialog';
import TokenizationDialog from './TokenizationDialog';

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

const initialBill: Bill = {
  id: '',
  creditorAccount: '',
  debtor: '',
  amount: 0,
  billImageUrl: '',
  contractImageUrl: '',
  tokenBatchCode: '',
  isCleared: false,
  isOnChain: false,
  createdAt: new Date(),
  createdWallet: '',
  updatedAt: new Date(),
  updatedWallet: '',
  dueDate: '',
};

const initialTokenizationForm: TokenizationForm = {
  stableCoin: 'USDT',
  minTerm: 1,
  maxTerm: 12,
  interestRate: 5,
  defaultRate: 10,
};

// 生成模拟数据
const generateMockData = (): Bill[] => {
  const bills: Bill[] = [];
  const billImages = [
    'https://example.com/bills/bill1.jpg',
    'https://example.com/bills/bill2.jpg',
    'https://example.com/bills/bill3.jpg',
    'https://example.com/bills/bill4.jpg',
    'https://example.com/bills/bill5.jpg',
  ];

  const contractImages = [
    'https://example.com/contracts/contract1.pdf',
    'https://example.com/contracts/contract2.pdf',
    'https://example.com/contracts/contract3.pdf',
    'https://example.com/contracts/contract4.pdf',
    'https://example.com/contracts/contract5.pdf',
  ];

  const generateAmount = () => {
    const min = 100000;
    const max = 10000000;
    return Math.floor(Math.random() * (max - min + 1)) + min;
  };

  const specifiedCreditor = '0x00d255bc0f5158cd6c65a173d29e92ace80da0ad';

  for (let i = 0; i < 10; i++) {
    const isOnChain = Math.random() > 0.3; 
    const tokenBatchCode = isOnChain ? `BATCH${String(i + 1).padStart(3, '0')}` : '';
    const amount = generateAmount();
    
    bills.push({
      id: `BILL${String(i + 1).padStart(4, '0')}`,
      creditorAccount: specifiedCreditor,
      debtor: `0x${Math.random().toString(16).slice(2, 42)}`,
      amount,
      billImageUrl: billImages[Math.floor(Math.random() * billImages.length)],
      contractImageUrl: contractImages[Math.floor(Math.random() * contractImages.length)],
      tokenBatchCode,
      isCleared: isOnChain ? Math.random() > 0.7 : false,
      isOnChain,
      createdAt: new Date(Date.now() - Math.floor(Math.random() * 30) * 24 * 60 * 60 * 1000),
      createdWallet: specifiedCreditor,
      updatedAt: new Date(),
      updatedWallet: specifiedCreditor,
      dueDate: '',
    });
  }

  return bills;
};

const BillManagement: React.FC = () => {
  const [bills, setBills] = useState<Bill[]>(generateMockData());
  const [open, setOpen] = useState(false);
  const [currentBill, setCurrentBill] = useState<Bill>(initialBill);
  const [isEditing, setIsEditing] = useState(false);
  const [detailOpen, setDetailOpen] = useState(false);
  const [detailBill, setDetailBill] = useState<Bill | null>(null);
  const [rowSelection, setRowSelection] = useState<Record<string, boolean>>({});
  const [sorting, setSorting] = useState<SortingState>([]);
  const [globalFilter, setGlobalFilter] = useState('');
  const [currentWallet, setCurrentWallet] = useState<{ address: string; type: string } | null>(null);
  const [onChainSuccessOpen, setOnChainSuccessOpen] = useState(false);
  const [onChainSuccessBillId, setOnChainSuccessBillId] = useState('');
  const [tokenizationOpen, setTokenizationOpen] = useState(false);
  const [tokenizationForm, setTokenizationForm] = useState<TokenizationForm>(initialTokenizationForm);
  const [selectedBillsForTokenization, setSelectedBillsForTokenization] = useState<Bill[]>([]);
  
  // 搜索条件
  const [searchCriteria, setSearchCriteria] = useState({
    debtor: '',
    isOnChain: '',
    tokenBatchCode: '',
  });

  // 监听钱包连接状态
  useEffect(() => {
    const walletService = WalletService.getInstance();
    
    // 立即检查一次钱包状态
    const checkWallet = () => {
      const wallet = walletService.getCurrentWallet();
       console.log('Current wallet => ', wallet); //这个会不停的执行， 好像不是我要的
      if (!wallet) {
        console.log('No wallet connected');
        setCurrentWallet(null);
        return;
      }
      setCurrentWallet(wallet);
    };

    // 初始检查
    checkWallet();

    // 监听钱包状态变化
    const interval = setInterval(checkWallet, 1000);

    // 清理函数
    return () => {
      clearInterval(interval);
    };
  }, []);

  // 过滤数据
  const filteredBills = useMemo(() => {
    console.log('Filtering bills with wallet:', currentWallet); 
    return bills.filter(bill => {
      // 首先检查是否已连接钱包
      if (!currentWallet) {
        console.log('No wallet connected'); 
        return false;
      }

      // 检查债权人是否匹配当前连接的钱包地址
      const isCreditor = bill.creditorAccount.toLowerCase() === currentWallet.address.toLowerCase();
      console.log('Bill creditor:', bill.creditorAccount, 'Current wallet:', currentWallet.address, 'Is creditor:', isCreditor); 
      
      // 其他过滤条件
      const matchDebtor = bill.debtor.toLowerCase().includes(searchCriteria.debtor.toLowerCase());
      const matchOnChain = searchCriteria.isOnChain === '' || 
        (searchCriteria.isOnChain === 'true' && bill.isOnChain) ||
        (searchCriteria.isOnChain === 'false' && !bill.isOnChain);
      const matchTokenBatch = bill.tokenBatchCode.toLowerCase().includes(searchCriteria.tokenBatchCode.toLowerCase());
      
      return isCreditor && matchDebtor && matchOnChain && matchTokenBatch;
    });
  }, [bills, searchCriteria, currentWallet]);

  const handleSearchChange = (field: string, value: string) => {
    setSearchCriteria(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const handleReset = () => {
    setSearchCriteria({
      debtor: '',
      isOnChain: '',
      tokenBatchCode: '',
    });
    setGlobalFilter('');
  };

  const columnHelper = createColumnHelper<Bill>();

  const columns = useMemo(() => [
    columnHelper.display({
      id: 'select',
      header: ({ table }) => (
        <Checkbox
          checked={table.getIsAllRowsSelected()}
          indeterminate={table.getIsSomeRowsSelected()}
          onChange={table.getToggleAllRowsSelectedHandler()}
        />
      ),
      cell: ({ row }) => (
        <Checkbox
          checked={row.getIsSelected()}
          // 仅当 tokenBatchCode 为空，复选框才可用
          disabled={row.original.tokenBatchCode !== ''}
          onChange={row.getToggleSelectedHandler()}
        />
      ),
    }),
    columnHelper.accessor('id', {
      header: '票据号',
      cell: info => (
        <Link
          component="button"
          variant="body2"
          onClick={() => handleViewDetail(info.row.original)}
          sx={{ textDecoration: 'none' }}
        >
          {info.getValue()}
        </Link>
      ),
    }),
    columnHelper.accessor('debtor', {
      header: '债务人',
    }),
    columnHelper.accessor('amount', {
      header: '金额',
      cell: info => `${info.getValue().toLocaleString()} 元`,
    }),
    columnHelper.accessor('tokenBatchCode', {
      header: 'Token批次号',
    }),
    columnHelper.accessor(row => row, {
      id: 'status',
      header: '状态',
      cell: info => (
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Chip
            label={info.getValue().isOnChain ? '已上链' : '未上链'}
            color={info.getValue().isOnChain ? 'success' : 'default'}
            size="small"
          />
          <Chip
            label={info.getValue().isCleared ? '已清算' : '未清算'}
            color={info.getValue().isCleared ? 'primary' : 'default'}
            size="small"
          />
        </Box>
      ),
    }),
    columnHelper.accessor(row => row, {
      id: 'actions',
      header: '操作',
      cell: info => (
        <Box>
          <Tooltip title="查看详情">
            <IconButton
              size="small"
              onClick={() => handleViewDetail(info.getValue())}
            >
              <VisibilityIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="编辑">
            <IconButton
              size="small"
              onClick={() => handleEdit(info.getValue())}
              disabled={info.getValue().isOnChain}
            >
              <EditIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="删除">
            <IconButton
              size="small"
              onClick={() => handleDelete(info.getValue().id)}
              disabled={info.getValue().isOnChain}
            >
              <DeleteIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="上链">
            <IconButton
              size="small"
              onClick={() => handleOnChain(info.getValue())}
              disabled={info.getValue().isOnChain}
              color="primary"
            >
              <LinkIcon />
            </IconButton>
          </Tooltip>
        </Box>
      ),
    }),
  ], []);

  const table = useReactTable({
    data: filteredBills,
    columns,
    state: {
      sorting,
      rowSelection,
      globalFilter,
    },
    onSortingChange: setSorting as OnChangeFn<SortingState>,
    onRowSelectionChange: setRowSelection,
    onGlobalFilterChange: setGlobalFilter,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    enableRowSelection: true,
  });

  const handleViewDetail = (bill: Bill) => {
    setDetailBill(bill);
    setDetailOpen(true);
  };

  const handleCloseDetail = () => {
    setDetailOpen(false);
    setDetailBill(null);
  };

  const handleOpen = () => {
    setOpen(true);
    setCurrentBill(initialBill);
    setIsEditing(false);
  };

  const handleClose = () => {
    setOpen(false);
  };

  const handleEdit = (bill: Bill) => {
    if (bill.isOnChain) {
      alert('已上链的票据不允许修改');
      return;
    }
    setCurrentBill(bill);
    setIsEditing(true);
    setOpen(true);
  };

  const handleDelete = (id: string) => {
    const bill = bills.find(b => b.id === id);
    if (bill?.isOnChain) {
      alert('已上链的票据不允许删除');
      return;
    }
    setBills(bills.filter(bill => bill.id !== id));
  };

  const handleSubmit = () => {
    if (isEditing) {
      setBills(bills.map(bill => 
        bill.id === currentBill.id ? currentBill : bill
      ));
    } else {
      setBills([...bills, { ...currentBill, id: Date.now().toString() }]);
    }
    handleClose();
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value, type, checked } = e.target;
    setCurrentBill(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value
    }));
  };

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>, type: 'bill' | 'contract') => {
    const file = e.target.files?.[0];
    if (file) {
      // 这里应该实现文件上传逻辑，现在只是模拟
      const url = URL.createObjectURL(file);
      setCurrentBill(prev => ({
        ...prev,
        [type === 'bill' ? 'billImageUrl' : 'contractImageUrl']: url
      }));
    }
  };

  const handleOnChain = (bill: Bill) => {
    // 这里添加上链逻辑
    setBills(bills.map(b => 
      b.id === bill.id ? { ...b, isOnChain: true } : b
    ));
    // 显示上链成功提示
    setOnChainSuccessBillId(bill.id);
    setOnChainSuccessOpen(true);
  };

  const handleCloseOnChainSuccess = () => {
    setOnChainSuccessOpen(false);
    setOnChainSuccessBillId('');
  };

  const handleTokenization = () => {
    console.log('rowSelection:', rowSelection); // 打印 rowSelection 状态
    console.log('filteredBills:', filteredBills); // 打印 filteredBills 状态
    filteredBills.forEach(bill => {
      console.log(`Bill ${bill.id} isOnChain: ${bill.isOnChain}, tokenBatchCode: ${bill.tokenBatchCode}`); // 打印每个票据的 isOnChain 和 tokenBatchCode 状态
    });

    const selectedBills = filteredBills.filter(bill => 
      rowSelection[bill.id] && 
      bill.isOnChain && 
      !bill.tokenBatchCode
    );
    console.log('selectedBills:', selectedBills); // 打印选中的票据
    
    // if (selectedBills.length === 0) {
    //   alert('请选择已上链且未代币化的票据');
    //   return;
    // }
    
    setSelectedBillsForTokenization(selectedBills);
    setTokenizationOpen(true);
  };

  const handleCloseTokenization = () => {
    setTokenizationOpen(false);
    setTokenizationForm(initialTokenizationForm);
    setSelectedBillsForTokenization([]);
  };

  const handleTokenizationSubmit = () => {
    // 这里添加代币化逻辑
    const batchCode = `BATCH${Date.now()}`;
    setBills(bills.map(bill => 
      selectedBillsForTokenization.some(selected => selected.id === bill.id)
        ? { ...bill, tokenBatchCode: batchCode }
        : bill
    ));
    handleCloseTokenization();
  };

  const handleTokenizationFormChange = (field: keyof TokenizationForm, value: string | number) => {
    setTokenizationForm(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const totalAmount = selectedBillsForTokenization.reduce((sum, bill) => sum + bill.amount, 0);

  return (
    <Layout>
      <Box sx={{ p: 3 }}>
        <Grid container justifyContent="space-between" alignItems="center" mb={3}>
          <Typography variant="h5" component="h2">
            票据管理
          </Typography>
          <Box>
            <Button
              variant="contained"
              color="secondary"
              onClick={handleTokenization}
              sx={{ mr: 2 }}
            >
              代币化
            </Button>
            <Button
              variant="contained"
              color="primary"
              startIcon={<AddIcon />}
              onClick={handleOpen}
              disabled={!currentWallet}
            >
              新增票据
            </Button>
          </Box>
        </Grid>

        {!currentWallet && (
          <Alert severity="warning" sx={{ mb: 3 }}>
            请先连接钱包以查看您的票据
          </Alert>
        )}

        {/* 搜索工具栏 */}
        <Paper sx={{ p: 2, mb: 2 }}>
          <Grid container spacing={2} alignItems="center">
            <Grid item xs={12} sm={4}>
              <TextField
                fullWidth
                size="small"
                label="债务人"
                value={searchCriteria.debtor}
                onChange={(e) => handleSearchChange('debtor', e.target.value)}
                InputProps={{
                  startAdornment: <SearchIcon sx={{ mr: 1, color: 'text.secondary' }} />,
                }}
              />
            </Grid>
            <Grid item xs={12} sm={4}>
              <FormControl fullWidth size="small">
                <InputLabel>上链状态</InputLabel>
                <Select
                  value={searchCriteria.isOnChain}
                  label="上链状态"
                  onChange={(e) => handleSearchChange('isOnChain', e.target.value)}
                >
                  <MenuItem value="">全部</MenuItem>
                  <MenuItem value="true">已上链</MenuItem>
                  <MenuItem value="false">未上链</MenuItem>
                </Select>
              </FormControl>
            </Grid>
            <Grid item xs={12} sm={4}>
              <TextField
                fullWidth
                size="small"
                label="票据批次"
                value={searchCriteria.tokenBatchCode}
                onChange={(e) => handleSearchChange('tokenBatchCode', e.target.value)}
                InputProps={{
                  startAdornment: <SearchIcon sx={{ mr: 1, color: 'text.secondary' }} />,
                }}
              />
            </Grid>
          </Grid>
        </Paper>

        {/* 数据表格 */}
        <Paper sx={{ width: '100%', mb: 2 }}>
          <TableContainer>
            <Table>
              <TableHead>
                {table.getHeaderGroups().map(headerGroup => (
                  <TableRow key={headerGroup.id}>
                    {headerGroup.headers.map(header => (
                      <TableCell
                        key={header.id}
                        onClick={header.column.getToggleSortingHandler()}
                        sx={{ cursor: 'pointer' }}
                      >
                        {flexRender(
                          header.column.columnDef.header,
                          header.getContext()
                        )}
                      </TableCell>
                    ))}
                  </TableRow>
                ))}
              </TableHead>
              <TableBody>
                {table.getRowModel().rows.map(row => (
                  <TableRow 
                    key={row.id}
                    selected={row.getIsSelected()}
                    hover
                  >
                    {row.getVisibleCells().map(cell => (
                      <TableCell key={cell.id}>
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext()
                        )}
                      </TableCell>
                    ))}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
          <Box sx={{ p: 2, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <Box>
              <Button
                onClick={() => table.setPageIndex(0)}
                disabled={!table.getCanPreviousPage()}
              >
                首页
              </Button>
              <Button
                onClick={() => table.previousPage()}
                disabled={!table.getCanPreviousPage()}
              >
                上一页
              </Button>
              <Button
                onClick={() => table.nextPage()}
                disabled={!table.getCanNextPage()}
              >
                下一页
              </Button>
              <Button
                onClick={() => table.setPageIndex(table.getPageCount() - 1)}
                disabled={!table.getCanNextPage()}
              >
                末页
              </Button>
            </Box>
            <Box>
              <Typography>
                第 {table.getState().pagination.pageIndex + 1} 页，共 {table.getPageCount()} 页
              </Typography>
            </Box>
          </Box>
        </Paper>

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

        {/* 上链成功提示对话框 */}
        <Dialog
          open={onChainSuccessOpen}
          onClose={handleCloseOnChainSuccess}
          maxWidth="xs"
          fullWidth
        >
          <DialogTitle>上链成功</DialogTitle>
          <DialogContent>
            <Box sx={{ pt: 2, textAlign: 'center' }}>
              <CheckCircleIcon color="success" sx={{ fontSize: 48, mb: 2 }} />
              <Typography variant="h6">
                票据编号 {onChainSuccessBillId} 已上链
              </Typography>
            </Box>
          </DialogContent>
          <DialogActions>
            <Button onClick={handleCloseOnChainSuccess} color="primary">
              确定
            </Button>
          </DialogActions>
        </Dialog>

        <AddBillDialog
          open={open}
          onClose={handleClose}
          onSubmit={handleSubmit}
          currentBill={currentBill}
          isEditing={isEditing}
          handleInputChange={handleInputChange}
          handleFileUpload={handleFileUpload}
          initialBill={initialBill}
        />

        <TokenizationDialog
          open={tokenizationOpen}
          onClose={handleCloseTokenization}
          onSubmit={handleTokenizationSubmit}
          selectedBillsForTokenization={selectedBillsForTokenization}
          tokenizationForm={tokenizationForm}
          handleTokenizationFormChange={handleTokenizationFormChange}
        />
      </Box>
    </Layout>
  );
};


export default BillManagement; 