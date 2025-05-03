import React, { useState } from 'react';
import { Button, TextField, Typography, Box, Paper, CircularProgress } from '@mui/material';
import AuthService from '../services/authService';
import ApiService from '../services/apiService';

const ApiTestComponent: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string>('');
  const [address, setAddress] = useState<string>('');
  const [error, setError] = useState<string | null>(null);

  const handleAddressChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setAddress(e.target.value);
  };

  const testChallengeEndpoint = async () => {
    setLoading(true);
    setError(null);
    try {
      const authService = AuthService.getInstance();
      const testAddress = address || '0x123456789012345678901234567890123456789';
      
      setResult(`Sending challenge request for address: ${testAddress}`);
      
      const challengeResponse = await authService.requestChallenge(testAddress);
      setResult(JSON.stringify(challengeResponse, null, 2));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(`Error: ${errorMessage}`);
      setResult('');
    } finally {
      setLoading(false);
    }
  };

  const testDirectApiCall = async () => {
    setLoading(true);
    setError(null);
    try {
      const testAddress = address || '0x123456789012345678901234567890123456789';
      
      setResult(`Sending direct API request for challenge to /rwa/user/challenge`);
      
      // Direct fetch call to test proxy
      const response = await fetch('/rwa/user/challenge', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ address: testAddress }),
      });
      
      const data = await response.json();
      setResult(JSON.stringify(data, null, 2));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(`Error: ${errorMessage}`);
      setResult('');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Paper elevation={3} sx={{ p: 3, m: 2, maxWidth: 600, mx: 'auto' }}>
      <Typography variant="h5" gutterBottom>
        API Connection Test
      </Typography>
      
      <Box sx={{ mb: 2 }}>
        <TextField
          label="Wallet Address (optional)"
          variant="outlined"
          fullWidth
          value={address}
          onChange={handleAddressChange}
          placeholder="0x..."
          sx={{ mb: 2 }}
        />
        
        <Box sx={{ display: 'flex', gap: 2 }}>
          <Button 
            variant="contained" 
            onClick={testChallengeEndpoint}
            disabled={loading}
          >
            Test AuthService Challenge
          </Button>
          
          <Button 
            variant="outlined" 
            onClick={testDirectApiCall}
            disabled={loading}
          >
            Test Direct API Call
          </Button>
        </Box>
      </Box>
      
      {loading && (
        <Box sx={{ display: 'flex', justifyContent: 'center', my: 2 }}>
          <CircularProgress />
        </Box>
      )}
      
      {error && (
        <Typography color="error" sx={{ my: 2, p: 2, bgcolor: 'rgba(255,0,0,0.1)', borderRadius: 1 }}>
          {error}
        </Typography>
      )}
      
      {result && (
        <Box sx={{ mt: 2 }}>
          <Typography variant="subtitle1">Result:</Typography>
          <pre style={{ 
            background: '#f5f5f5', 
            padding: '10px', 
            borderRadius: '4px',
            overflow: 'auto',
            maxHeight: '300px'
          }}>
            {result}
          </pre>
        </Box>
      )}
    </Paper>
  );
};

export default ApiTestComponent; 