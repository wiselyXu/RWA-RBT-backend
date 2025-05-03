import React from 'react';
import { Box, Typography } from '@mui/material';

const Footer: React.FC = () => {
  return (
    <Box sx={{ bgcolor: 'background.paper', p: 6, textAlign: 'center' }}>
      <Typography variant="body2" color="text.secondary">
        © 2025 区块链票据融资平台. 保留所有权利.
      </Typography>
    </Box>
  );
};

export default Footer;
