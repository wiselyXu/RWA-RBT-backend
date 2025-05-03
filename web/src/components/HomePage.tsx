import React, { useState } from 'react';
import { Typography, Box, Container, Grid, Card, CardContent, Avatar, Paper, Button, Switch, FormControlLabel } from '@mui/material';
import { DataObject, Business, Security, VerifiedUser } from '@mui/icons-material';
import Layout from './Layout';
import ApiTestComponent from './ApiTestComponent';

const businessCards = [
  {
    icon: <Business fontSize="large" color="primary" />,
    title: '企业入驻',
    desc: '企业实名认证后可发行和融资票据，流程合规高效。',
  },
  {
    icon: <DataObject fontSize="large" color="primary" />,
    title: '票据市场',
    desc: '票据可自由流转、购买、融资，市场透明公开。',
  },
  {
    icon: <Security fontSize="large" color="primary" />,
    title: '区块链安全',
    desc: '基于区块链技术，数据不可篡改，交易全程可追溯。',
  },
  {
    icon: <VerifiedUser fontSize="large" color="primary" />,
    title: '金融合规',
    desc: '严格风控体系，保障票据融资合规与安全。',
  },
];

const marketPreview = [
  { creditor: '企业A', batch: 'RBT202401', amount: 1000000, repay: '2025-04-01' },
  { creditor: '企业B', batch: 'RBT202402', amount: 800000, repay: '2025-05-01' },
  { creditor: '企业C', batch: 'RBT202403', amount: 1200000, repay: '2025-06-01' },
];

const advantages = [
  { icon: <Security color="primary" />, title: '安全透明', desc: '区块链保障数据安全与透明' },
  { icon: <Business color="primary" />, title: '高效撮合', desc: '智能合约自动撮合交易' },
  { icon: <VerifiedUser color="primary" />, title: '合规风控', desc: '严格合规，风控体系完善' },
];

const HomePage: React.FC = () => {
  const [showDebug, setShowDebug] = useState(false);

  const toggleDebug = () => {
    setShowDebug(!showDebug);
  };

  return (
    <Layout>
      {/* Banner 区块 */}
      <Box sx={{ bgcolor: 'linear-gradient(90deg, #1976d2 0%, #42a5f5 100%)', py: 8, textAlign: 'center', color: '#fff', background: 'linear-gradient(90deg, #1976d2 0%, #42a5f5 100%)' }}>
        <Container maxWidth="md">
          <Typography variant="h3" fontWeight={700} mb={2} color="#fff">
            区块链票据融资平台
          </Typography>
          <Typography variant="h6" mb={4} color="#e3f2fd">
            安全 · 高效 · 透明的票据融资服务
          </Typography>
          <Button variant="contained" size="large" color="secondary" sx={{ fontWeight: 700, px: 5 }}>
            立即体验
          </Button>
        </Container>
      </Box>

      {/* Debug Switch */}
      <Box sx={{ display: 'flex', justifyContent: 'flex-end', p: 2 }}>
        <FormControlLabel
          control={<Switch checked={showDebug} onChange={toggleDebug} />}
          label="显示API调试工具"
        />
      </Box>
      
      {/* API Test Component (only shown when debug is enabled) */}
      {showDebug && <ApiTestComponent />}

      {/* 业务介绍 */}
      <Container maxWidth="lg" sx={{ mt: 6 }}>
        <Grid container spacing={3}>
          {businessCards.map((card, idx) => (
            <Grid item xs={12} sm={6} md={3} key={idx}>
              <Card sx={{ textAlign: 'center', py: 4, px: 2, borderRadius: 3, boxShadow: 2 }}>
                <Avatar sx={{ bgcolor: '#e3f2fd', mx: 'auto', mb: 2, width: 56, height: 56 }}>
                  {card.icon}
                </Avatar>
                <Typography variant="h6" fontWeight={700} mb={1}>{card.title}</Typography>
                <Typography variant="body2" color="text.secondary">{card.desc}</Typography>
              </Card>
            </Grid>
          ))}
        </Grid>
      </Container>

      {/* 票据市场预览 */}
      <Container maxWidth="md" sx={{ mt: 8 }}>
        <Paper sx={{ p: 3, borderRadius: 3, boxShadow: 1 }}>
          <Typography variant="h6" fontWeight={700} mb={2} color="primary">票据市场预览</Typography>
          <Grid container spacing={2}>
            {marketPreview.map((item, idx) => (
              <Grid item xs={12} sm={4} key={idx}>
                <Card sx={{ p: 2, borderRadius: 2, boxShadow: 1, textAlign: 'center' }}>
                  <Typography variant="subtitle1" fontWeight={700}>{item.creditor}</Typography>
                  <Typography variant="body2" color="text.secondary">批次：{item.batch}</Typography>
                  <Typography variant="body2" color="text.secondary">金额：{item.amount.toLocaleString()} 元</Typography>
                  <Typography variant="body2" color="text.secondary">最后偿还日：{item.repay}</Typography>
                  <Button variant="outlined" color="primary" size="small" sx={{ mt: 1 }}>购买</Button>
                </Card>
              </Grid>
            ))}
          </Grid>
        </Paper>
      </Container>

      {/* 平台优势 */}
      <Container maxWidth="lg" sx={{ mt: 8, mb: 8 }}>
        <Typography variant="h6" fontWeight={700} mb={3} color="primary">平台优势</Typography>
        <Grid container spacing={3}>
          {advantages.map((adv, idx) => (
            <Grid item xs={12} sm={4} key={idx}>
              <Card sx={{ textAlign: 'center', py: 4, px: 2, borderRadius: 3, boxShadow: 1 }}>
                <Avatar sx={{ bgcolor: '#e3f2fd', mx: 'auto', mb: 2, width: 56, height: 56 }}>
                  {adv.icon}
                </Avatar>
                <Typography variant="subtitle1" fontWeight={700} mb={1}>{adv.title}</Typography>
                <Typography variant="body2" color="text.secondary">{adv.desc}</Typography>
              </Card>
            </Grid>
          ))}
        </Grid>
      </Container>
    </Layout>
  );
};

export default HomePage; 