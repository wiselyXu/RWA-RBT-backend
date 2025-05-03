import React from 'react';
import { Box } from '@mui/material';
import InvoiceBatchList from '../components/InvoiceBatchList';

const InvoiceBatchPage: React.FC = () => {
  return (
    <Box sx={{ 
      flexGrow: 1,
      display: 'flex',
      flexDirection: 'column',
      minHeight: 'calc(100vh - 64px)', // Assuming app bar height is 64px
      bgcolor: 'background.default'
    }}>
      <Box component="main" sx={{ flexGrow: 1, p: 0 }}>
        <InvoiceBatchList />
      </Box>
    </Box>
  );
};

export default InvoiceBatchPage; 