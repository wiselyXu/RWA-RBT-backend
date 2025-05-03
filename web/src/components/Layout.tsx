import React from 'react';
import { Box } from '@mui/material';
import Header from './Header';
import Footer from './Footer';

interface LayoutProps {
  children: React.ReactNode;
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  return (
    <Box 
      sx={{
        display: 'flex',
        flexDirection: 'column',
        minHeight: '100vh', // 确保容器至少占满视口高度
      }}
    >
      <Header />
      <Box 
        component="main" 
        sx={{
          flexGrow: 1, // 让内容区域自动填充剩余空间
        }}
      >
        {children}
      </Box>
      <Footer />
    </Box>
  );
};

export default Layout; 