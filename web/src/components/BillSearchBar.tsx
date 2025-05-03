import React from 'react';
import { Paper, Grid, TextField, FormControl, InputLabel, Select, MenuItem, Button } from '@mui/material';
import { Search as SearchIcon } from '@mui/icons-material';

interface Props {
  searchCriteria: {
    debtor: string;
    isOnChain: string;
    tokenBatchCode: string;
  };
  handleSearchChange: (field: string, value: string) => void;
  handleReset: () => void;
}

const BillSearchBar: React.FC<Props> = ({ searchCriteria, handleSearchChange, handleReset }) => {
  return (
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
            label="Token批次"
            value={searchCriteria.tokenBatchCode}
            onChange={(e) => handleSearchChange('tokenBatchCode', e.target.value)}
            InputProps={{
              startAdornment: <SearchIcon sx={{ mr: 1, color: 'text.secondary' }} />,
            }}
          />
        </Grid>
        <Grid item xs={12} sx={{ textAlign: 'right' }}>
          <Button variant="contained" onClick={handleReset}>
            重置
          </Button>
        </Grid>
      </Grid>
    </Paper>
  );
};

export default BillSearchBar;

// Add an empty export statement to ensure it's treated as a module
export {};