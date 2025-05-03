import React, { useMemo } from 'react';
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableContainer, 
  TableHead, 
  TableRow, 
  Paper, 
  IconButton, 
  Link, 
  Chip, 
  Checkbox, 
  Box, 
  Button, 
  Typography 
} from '@mui/material';
import { Edit as EditIcon, Delete as DeleteIcon } from '@mui/icons-material';
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
  RowSelectionState // Import RowSelectionState
} from '@tanstack/react-table';

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
  filteredBills: Bill[];
  handleViewDetail: (bill: Bill) => void;
  handleEdit: (bill: Bill) => void;
  handleDelete: (id: string) => void;
  sorting: SortingState;
  setSorting: OnChangeFn<SortingState>;
  rowSelection: RowSelectionState; // Use RowSelectionState
  setRowSelection: OnChangeFn<RowSelectionState>; // Use OnChangeFn<RowSelectionState>
  globalFilter: string;
  setGlobalFilter: (globalFilter: string) => void;
}

const BillTable: React.FC<Props> = ({ 
  filteredBills, 
  handleViewDetail, 
  handleEdit, 
  handleDelete, 
  sorting, 
  setSorting, 
  rowSelection, 
  setRowSelection, 
  globalFilter, 
  setGlobalFilter 
}) => {
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
          disabled={row.original.isOnChain}
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
          <IconButton
            onClick={() => handleEdit(info.getValue())}
            color="primary"
            disabled={info.getValue().isOnChain}
            size="small"
          >
            <EditIcon />
          </IconButton>
          <IconButton
            onClick={() => handleDelete(info.getValue().id)}
            color="error"
            disabled={info.getValue().isOnChain}
            size="small"
          >
            <DeleteIcon />
          </IconButton>
        </Box>
      ),
    }),
  ], [handleViewDetail, handleEdit, handleDelete]);

  const table = useReactTable({
    data: filteredBills,
    columns,
    state: {
      sorting,
      rowSelection,
      globalFilter,
    },
    onSortingChange: setSorting,
    onRowSelectionChange: setRowSelection,
    onGlobalFilterChange: setGlobalFilter,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    enableRowSelection: true,
  });

  return (
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
  );
};

export default BillTable;
