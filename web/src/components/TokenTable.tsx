import React, { useState, useMemo } from 'react';
import {
  Box,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Checkbox,
  FormControlLabel,
  MenuItem,
  Menu,
  IconButton,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
} from '@mui/material';
import { ArrowUpward, ArrowDownward, Settings, FilterList, Sort, Info } from '@mui/icons-material';
import TokenDetailDialog from './TokenDetailDialog';
import { Link } from 'react-router-dom';

type TokenData = {
  tokenBatchNumber: string;
  creditorAccount: string;
  debtor: string;
  stablecoin: string;
  billQuantity: number;
  issuedAmount: bigint;
  isDebtorSigned: boolean;
  createTime: Date;
  createWallet: string;
  modifyTime: Date;
  modifyWallet: string;
};

type Column = {
  id: keyof TokenData | 'actions';
  label: string;
  hidden?: boolean;
  sortable?: boolean;
  filterable?: boolean;
};

interface TokenTableProps {
  data: TokenData[];
}

const TokenTable: React.FC<TokenTableProps> = ({ data }) => {
  const [columns, setColumns] = useState<Column[]>([
    { id: 'tokenBatchNumber', label: 'token 批次编号', sortable: true, filterable: true },
    { id: 'creditorAccount', label: '债权人账户', sortable: true, filterable: true },
    { id: 'debtor', label: '债务方', sortable: true, filterable: true },
    { id: 'stablecoin', label: '稳定币', sortable: true, filterable: true },
    { id: 'billQuantity', label: '票据数量', sortable: true, filterable: true },
    { id: 'issuedAmount', label: '发行币额', sortable: true, filterable: true },
    { id: 'isDebtorSigned', label: '债务人是否签名', sortable: true, filterable: true },
    { id: 'createTime', label: '创建时间', sortable: true, filterable: true, hidden: true },
    { id: 'createWallet', label: '创建钱包', sortable: true, filterable: true, hidden: true },
    { id: 'modifyTime', label: '修改时间', sortable: true, filterable: true, hidden: true },
    { id: 'modifyWallet', label: '修改钱包', sortable: true, filterable: true, hidden: true },
    { id: 'actions', label: '操作', hidden: false },
  ]);

  const [sortColumn, setSortColumn] = useState<keyof TokenData | null>(null);
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
  const [filters, setFilters] = useState<{ [key in keyof TokenData]?: string }>({});
  const [openDialog, setOpenDialog] = useState(false);
  const [anchorElFilter, setAnchorElFilter] = useState<null | HTMLElement>(null);
  const [filterColumn, setFilterColumn] = useState<keyof TokenData | null>(null);
  const [selectedToken, setSelectedToken] = useState<TokenData | null>(null);
  const [showDetailDialog, setShowDetailDialog] = useState(false);

  const handleSort = (columnId: keyof TokenData) => {
    if (sortColumn === columnId) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortColumn(columnId);
      setSortDirection('asc');
    }
  };

  const handleFilterChange = (columnId: keyof TokenData, value: string) => {
    setFilters((prevFilters) => ({ ...prevFilters, [columnId]: value }));
  };

  const handleColumnVisibilityChange = (columnId: keyof TokenData | 'actions', checked: boolean) => {
    setColumns((prevColumns) =>
      prevColumns.map((column) =>
        column.id === columnId ? { ...column, hidden: !checked } : column
      )
    );
  };

  const handleDialogOpen = () => {
    setOpenDialog(true);
  };

  const handleDialogClose = () => {
    setOpenDialog(false);
  };

  const handleFilterClick = (event: React.MouseEvent<HTMLElement>, columnId: keyof TokenData) => {
    setAnchorElFilter(event.currentTarget);
    setFilterColumn(columnId);
  };

  const handleFilterClose = () => {
    setAnchorElFilter(null);
    setFilterColumn(null);
  };

  const handleTokenClick = (token: TokenData) => {
    setSelectedToken(token);
    setShowDetailDialog(true);
  };

  const handleDetailDialogClose = () => {
    setShowDetailDialog(false);
  };

  const filteredData = useMemo(() => {
    return data.filter((item) => {
      return Object.entries(filters).every(([key, value]) => {
        const itemValue = item[key as keyof TokenData];
        if (typeof itemValue === 'string') {
          return itemValue.toLowerCase().includes(value.toLowerCase());
        }
        if (typeof itemValue === 'number') {
          return String(itemValue).includes(value);
        }
        if (typeof itemValue === 'boolean') {
          return value === 'true' ? itemValue : value === 'false' ? !itemValue : true;
        }
        if (itemValue instanceof Date) {
          return itemValue.toLocaleDateString().includes(value);
        }
        if (typeof itemValue === 'bigint') {
          return String(itemValue).includes(value);
        }
        return true;
      });
    });
  }, [data, filters]);

  const sortedData = useMemo(() => {
    if (!sortColumn) return filteredData;
    return [...filteredData].sort((a, b) => {
      const valueA = a[sortColumn];
      const valueB = b[sortColumn];

      if (typeof valueA === 'string' && typeof valueB === 'string') {
        return sortDirection === 'asc' ? valueA.localeCompare(valueB) : valueB.localeCompare(valueA);
      }
      if (typeof valueA === 'number' && typeof valueB === 'number') {
        return sortDirection === 'asc' ? valueA - valueB : valueB - valueA;
      }
      if (typeof valueA === 'boolean' && typeof valueB === 'boolean') {
        return sortDirection === 'asc' ? (valueA === valueB ? 0 : valueA ? 1 : -1) : valueA === valueB ? 0 : valueA ? -1 : 1;
      }
      if (valueA instanceof Date && valueB instanceof Date) {
        return sortDirection === 'asc' ? valueA.getTime() - valueB.getTime() : valueB.getTime() - valueA.getTime();
      }
      if (typeof valueA === 'bigint' && typeof valueB === 'bigint') {
        return sortDirection === 'asc' ? Number(valueA - valueB) : Number(valueB - valueA);
      }
      return 0;
    });
  }, [filteredData, sortColumn, sortDirection]);

  return (
    <Box>
      {/* 设置按钮，点击打开弹窗 */}
      <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 2 }}>
        <Button variant="outlined" startIcon={<Settings />} onClick={handleDialogOpen}>
          设置列显示
        </Button>
      </Box>
      {/* 列显示设置弹窗 */}
      <Dialog open={openDialog} onClose={handleDialogClose}>
        <DialogTitle>设置列显示</DialogTitle>
        <DialogContent>
          {columns.map((column) => (
            <FormControlLabel
              key={column.id}
              control={
                <Checkbox
                  checked={!column.hidden}
                  onChange={(e) => handleColumnVisibilityChange(column.id, e.target.checked)}
                />
              }
              label={column.label}
            />
          ))}
        </DialogContent>
        <DialogActions>
          <Button onClick={handleDialogClose}>关闭</Button>
        </DialogActions>
      </Dialog>
      <TableContainer component={Paper}>
        <Table sx={{ minWidth: 650 }} aria-label="token table">
          <TableHead>
            <TableRow>
              {columns.map((column) => {
                if (column.hidden) return null;
                return (
                  <TableCell key={column.id}>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      {column.id !== 'actions' && column.sortable && (
                        <IconButton onClick={() => handleSort(column.id as keyof TokenData)} size="small" title="排序">
                          {sortColumn === column.id && sortDirection === 'asc' ? (
                            <ArrowUpward fontSize="inherit" />
                          ) : sortColumn === column.id && sortDirection === 'desc' ? (
                            <ArrowDownward fontSize="inherit" />
                          ) : <Sort fontSize="inherit" />}
                        </IconButton>
                      )}
                      {column.label}
                      {column.id !== 'actions' && column.filterable && (
                        <>
                          <IconButton
                            onClick={(e) => handleFilterClick(e, column.id as keyof TokenData)}
                            size="small"
                            title="过滤"
                          >
                            <FilterList fontSize="inherit" />
                          </IconButton>
                          <Menu
                            anchorEl={anchorElFilter}
                            open={Boolean(anchorElFilter) && filterColumn === column.id}
                            onClose={handleFilterClose}
                          >
                            <MenuItem value="" onClick={() => {
                              handleFilterChange(column.id as keyof TokenData, '');
                              handleFilterClose();
                            }}>
                              全部
                            </MenuItem>
                            {Array.from(new Set(sortedData.map((item) => item[column.id as keyof TokenData]))).map((value) => (
                              <MenuItem
                                key={String(value)}
                                value={String(value)}
                                onClick={() => {
                                  handleFilterChange(column.id as keyof TokenData, String(value));
                                  handleFilterClose();
                                }}
                              >
                                {String(value)}
                              </MenuItem>
                            ))}
                          </Menu>
                        </>
                      )}
                    </Box>
                  </TableCell>
                );
              })}
            </TableRow>
          </TableHead>
          <TableBody>
            {sortedData.map((row) => (
              <TableRow
                key={row.tokenBatchNumber}
                sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
              >
                {columns.map((column) => {
                  if (column.hidden) return null;
                  if (column.id === 'tokenBatchNumber') {
                    return (
                      <TableCell key={column.id} component="th" scope="row">
                        <Button
                          color="primary"
                          variant="text" // 确保按钮样式为文本样式
                          onClick={() => handleTokenClick(row)}
                        >
                          {row.tokenBatchNumber}
                        </Button>
                      </TableCell>
                    );
                  }
                  if (column.id === 'actions') {
                    return (
                      <TableCell key={column.id}>
                        <IconButton color="primary" aria-label="查看详情">
                          <Info />
                        </IconButton>
                      </TableCell>
                    );
                  }
                  const value = row[column.id as keyof TokenData];
                  return (
                    <TableCell key={column.id} component="th" scope="row">
                      {typeof value === 'boolean' ? (value ? '是' : '否') : typeof value === 'bigint' ? value.toString() : value instanceof Date ? value.toLocaleDateString() : value}
                    </TableCell>
                  );
                })}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
      {/* 使用 TokenDetailDialog 组件展示详情 */}
      <TokenDetailDialog
        open={showDetailDialog}
        onClose={handleDetailDialogClose}
        tokenData={selectedToken || undefined}
      />
    </Box>
  );
};

export default TokenTable;
