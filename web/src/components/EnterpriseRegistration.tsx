import React, { useState } from 'react';
import {
  Box,
  Button,
  TextField,
  Typography,
  Paper,
  InputLabel,
  FormControl,
  FormHelperText,
  CircularProgress,
} from '@mui/material';
import CloudUploadIcon from '@mui/icons-material/CloudUpload';
import Grid from '@mui/material/Grid';

interface FormData {
  companyName: string;
  creditCode: string;
  companyDesc: string;
  walletAddress: string;
  legalPersonName: string;
  legalPersonPhone: string;
  businessLicense: File | null;
  legalIdCard: File | null;
  companySeal: File | null;
  smsCode: string;
}

const initialFormData: FormData = {
  companyName: '',
  creditCode: '',
  companyDesc: '',
  walletAddress: '',
  legalPersonName: '',
  legalPersonPhone: '',
  businessLicense: null,
  legalIdCard: null,
  companySeal: null,
  smsCode: '',
};

const EnterpriseRegistration: React.FC = () => {
  const [form, setForm] = useState<FormData>(initialFormData);
  const [errors, setErrors] = useState<{ [key: string]: string }>({});
  const [submitting, setSubmitting] = useState(false);
  const [smsSent, setSmsSent] = useState(false);
  const [smsLoading, setSmsLoading] = useState(false);
  const [submitSuccess, setSubmitSuccess] = useState(false);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setForm((prev) => ({ ...prev, [name]: value }));
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, files } = e.target;
    if (files && files[0]) {
      setForm((prev) => ({ ...prev, [name]: files[0] }));
    }
  };

  const validate = () => {
    const newErrors: { [key: string]: string } = {};
    if (!form.companyName) newErrors.companyName = '企业名称不能为空';
    if (!form.creditCode) newErrors.creditCode = '统一信用号不能为空';
    if (!form.companyDesc) newErrors.companyDesc = '企业简介不能为空';
    if (!form.walletAddress) newErrors.walletAddress = '钱包地址不能为空';
    if (!form.legalPersonName) newErrors.legalPersonName = '法人姓名不能为空';
    if (!form.legalPersonPhone) newErrors.legalPersonPhone = '法人手机号不能为空';
    else if (!/^1[3-9]\d{9}$/.test(form.legalPersonPhone)) newErrors.legalPersonPhone = '手机号格式不正确';
    if (!form.businessLicense) newErrors.businessLicense = '请上传营业执照';
    if (!form.legalIdCard) newErrors.legalIdCard = '请上传法人身份证';
    if (!form.companySeal) newErrors.companySeal = '请上传企业签章';
    if (!form.smsCode) newErrors.smsCode = '请输入短信验证码';
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSendSms = () => {
    if (!form.legalPersonPhone || errors.legalPersonPhone) {
      setErrors((prev) => ({ ...prev, legalPersonPhone: '请填写正确的手机号' }));
      return;
    }
    setSmsLoading(true);
    setTimeout(() => {
      setSmsSent(true);
      setSmsLoading(false);
    }, 1000); // 模拟发送短信
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;
    setSubmitting(true);
    setTimeout(() => {
      setSubmitting(false);
      setSubmitSuccess(true);
    }, 1500); // 模拟提交
  };

  return (
    <Box maxWidth={700} mx="auto" mt={4}>
      <Paper elevation={3} sx={{ p: 4 }}>
        <Typography variant="h5" gutterBottom>
          企业入驻申请
        </Typography>
        <form onSubmit={handleSubmit} noValidate>
          <Grid container spacing={2}>
            <Grid item xs={12} sm={6}>
              <TextField
                label="企业名称"
                name="companyName"
                value={form.companyName}
                onChange={handleInputChange}
                fullWidth
                required
                error={!!errors.companyName}
                helperText={errors.companyName}
              />
            </Grid>
            <Grid item xs={12} sm={6}>
              <TextField
                label="统一信用号"
                name="creditCode"
                value={form.creditCode}
                onChange={handleInputChange}
                fullWidth
                required
                error={!!errors.creditCode}
                helperText={errors.creditCode}
              />
            </Grid>
            <Grid item xs={12}>
              <TextField
                label="企业简介"
                name="companyDesc"
                value={form.companyDesc}
                onChange={handleInputChange}
                fullWidth
                required
                multiline
                minRows={2}
                error={!!errors.companyDesc}
                helperText={errors.companyDesc}
              />
            </Grid>
            <Grid item xs={12} sm={6}>
              <TextField
                label="钱包地址"
                name="walletAddress"
                value={form.walletAddress}
                onChange={handleInputChange}
                fullWidth
                required
                error={!!errors.walletAddress}
                helperText={errors.walletAddress}
              />
            </Grid>
            <Grid item xs={12} sm={6}>
              <TextField
                label="法人姓名"
                name="legalPersonName"
                value={form.legalPersonName}
                onChange={handleInputChange}
                fullWidth
                required
                error={!!errors.legalPersonName}
                helperText={errors.legalPersonName}
              />
            </Grid>
            <Grid item xs={12} sm={8}>
              <TextField
                label="法人手机号"
                name="legalPersonPhone"
                value={form.legalPersonPhone}
                onChange={handleInputChange}
                fullWidth
                required
                error={!!errors.legalPersonPhone}
                helperText={errors.legalPersonPhone}
              />
            </Grid>
            <Grid item xs={12} sm={4}>
              <Button
                variant="outlined"
                fullWidth
                sx={{ height: '100%' }}
                onClick={handleSendSms}
                disabled={smsLoading || smsSent}
                startIcon={smsLoading ? <CircularProgress size={18} /> : null}
              >
                {smsSent ? '已发送' : '获取验证码'}
              </Button>
            </Grid>
            <Grid item xs={12} sm={6}>
              <TextField
                label="短信验证码"
                name="smsCode"
                value={form.smsCode}
                onChange={handleInputChange}
                fullWidth
                required
                error={!!errors.smsCode}
                helperText={errors.smsCode}
              />
            </Grid>
            {/* 图片上传区块 */}
            <Grid item xs={12} sm={6}>
              <FormControl fullWidth error={!!errors.businessLicense}>
                <InputLabel shrink>营业执照</InputLabel>
                <Button
                  variant="contained"
                  component="label"
                  startIcon={<CloudUploadIcon />}
                  sx={{ mt: 1 }}
                >
                  {form.businessLicense ? form.businessLicense.name : '上传营业执照'}
                  <input
                    type="file"
                    name="businessLicense"
                    accept="image/*,application/pdf"
                    hidden
                    onChange={handleFileChange}
                  />
                </Button>
                <FormHelperText>{errors.businessLicense}</FormHelperText>
              </FormControl>
            </Grid>
            <Grid item xs={12} sm={6}>
              <FormControl fullWidth error={!!errors.legalIdCard}>
                <InputLabel shrink>法人身份证</InputLabel>
                <Button
                  variant="contained"
                  component="label"
                  startIcon={<CloudUploadIcon />}
                  sx={{ mt: 1 }}
                >
                  {form.legalIdCard ? form.legalIdCard.name : '上传法人身份证'}
                  <input
                    type="file"
                    name="legalIdCard"
                    accept="image/*,application/pdf"
                    hidden
                    onChange={handleFileChange}
                  />
                </Button>
                <FormHelperText>{errors.legalIdCard}</FormHelperText>
              </FormControl>
            </Grid>
            <Grid item xs={12} sm={6}>
              <FormControl fullWidth error={!!errors.companySeal}>
                <InputLabel shrink>企业签章</InputLabel>
                <Button
                  variant="contained"
                  component="label"
                  startIcon={<CloudUploadIcon />}
                  sx={{ mt: 1 }}
                >
                  {form.companySeal ? form.companySeal.name : '上传企业签章'}
                  <input
                    type="file"
                    name="companySeal"
                    accept="image/*,application/pdf"
                    hidden
                    onChange={handleFileChange}
                  />
                </Button>
                <FormHelperText>{errors.companySeal}</FormHelperText>
              </FormControl>
            </Grid>
          </Grid>
          <Box mt={3}>
            <Button
              type="submit"
              variant="contained"
              color="primary"
              fullWidth
              disabled={submitting}
              size="large"
              sx={{ py: 1.5 }}
            >
              {submitting ? <CircularProgress size={24} /> : '提交申请'}
            </Button>
            {submitSuccess && (
              <Typography color="success.main" align="center" mt={2}>
                提交成功，等待审核！
              </Typography>
            )}
          </Box>
        </form>
      </Paper>
    </Box>
  );
};

export default EnterpriseRegistration; 