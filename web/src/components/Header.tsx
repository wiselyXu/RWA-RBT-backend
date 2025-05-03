import React, { useState } from 'react';
import {
  Box,
  Button,
  Menu,
  MenuItem,
  AppBar,
  Toolbar,
} from '@mui/material';
// 引入 Link 组件
import { Link } from 'react-router-dom';
// 导入 logo
import Logo from '../assets/logo.svg';
import WalletConnect from './WalletConnect';
import { useAuth, UserRole } from '../context/AuthContext';

// 定义菜单项接口
interface SubMenuItem {
  label: string;
  path: string;
}

interface MenuItem {
  label: string;
  path: string;
  subItems?: SubMenuItem[];
}

// 首页菜单数据 - 所有用户可见
const commonMenuItems: MenuItem[] = [
  {
    label: '首页',
    path: '/',
  },
  {
    label: 'Token市场',
    path: '/token-market',
  },
  {
    label: '我的token',
    path: '/my-tokens',
  },
];

// 企业用户菜单项 - 只有绑定了企业的用户可见
const enterpriseMenuItems: MenuItem[] = [
  {
    label: '我的债权',
    path: '/my-credits',
    subItems: [
      { label: '票据管理', path: '/my-credits/invoices' },
      { label: '批次管理', path: '/my-credits/batches' },
      { label: 'Token管理', path: '/my-credits/my-issued-tokens' }, 
    ],
  },
  {
    label: '我的债务',
    path: '/my-debts',
    subItems: [
      { label: '待我签名', path: '/my-debts/my-todolist' },
      { label: '偿还债务', path: '/my-debts/repay-debt' }, 
    ],
  },
];

const Header: React.FC = () => {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const { userInfo } = useAuth();

  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>, menuLabel: string) => {
    setAnchorEl(event.currentTarget);
    setActiveMenu(menuLabel);
  };

  const handleMenuClose = () => {
    setAnchorEl(null);
    setActiveMenu(null);
  };

  // 根据用户角色确定显示哪些菜单项
  const getMenuItems = (): MenuItem[] => {
    let menuItems = [...commonMenuItems];
    
    // 如果用户绑定了企业，添加企业菜单项
    if (userInfo?.isEnterpriseBound) {
      menuItems = [...menuItems, ...enterpriseMenuItems];
    }
    
    return menuItems;
  };

  return (
    <AppBar position="static" sx={{ backgroundColor: 'transparent', boxShadow: 'none' }}>
      <Toolbar sx={{ p: 0, display: 'flex', justifyContent: 'space-between' }}>
        {/* Logo 部分无背景 */}
        <Box sx={{ p: 1 }}>
          <Link to="/">
            <img src={Logo} alt="Logo" height="40" />
          </Link>
        </Box>
        
        {/* 渐变背景导航菜单 */}
        <Box
          sx={{
            display: { xs: 'none', md: 'flex' },
            flexGrow: 1,
            justifyContent: 'center',
            background: 'linear-gradient(90deg, #e3f2fd 0%, #1976d2 100%)',
            p: 1,
            mx: 2,
            borderRadius: '4px',
          }}
        >
          {getMenuItems().map((item) => (
            item.subItems ? (
              <React.Fragment key={item.label}>
                <Button 
                  color="inherit" 
                  onClick={(e) => handleMenuOpen(e, item.label)}
                >
                  {item.label}
                </Button>
                <Menu
                  anchorEl={anchorEl}
                  open={Boolean(anchorEl) && activeMenu === item.label}
                  onClose={handleMenuClose}
                >
                  {item.subItems.map((subItem) => (
                    <MenuItem
                      key={subItem.label}
                      component={Link}
                      to={subItem.path}
                      onClick={handleMenuClose}
                    >
                      {subItem.label}
                    </MenuItem>
                  ))}
                </Menu>
              </React.Fragment>
            ) : (
              <Button 
                key={item.label} 
                color="inherit" 
                component={Link} 
                to={item.path}
              >
                {item.label}
              </Button>
            )
          ))}
        </Box>
        
        {/* 钱包连接区域 */}
        <Box>
          <WalletConnect 
            position="relative" 
            top={0} 
            right={0} 
          />
        </Box>
      </Toolbar>
    </AppBar>
  );
};

export default Header;
