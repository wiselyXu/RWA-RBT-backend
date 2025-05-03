# RBT Frontend

This is the frontend for the RBT (Real-World Assets Token) platform.

## Setup and Running with API Proxy

To properly connect to the backend API at `http://127.0.0.1:8888/rwa`, follow these steps:

### Option 1: Using the start script

1. Make the start script executable:
   ```bash
   chmod +x start.sh
   ```

2. Run the start script:
   ```bash
   ./start.sh
   ```

This will automatically:
- Install the http-proxy-middleware package if needed
- Start the development server with the proper proxy configuration

### Option 2: Manual setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Start the development server:
   ```bash
   npm start
   ```

The proxy is configured in `src/setupProxy.js` to forward all `/rwa` requests to the backend at `http://127.0.0.1:8888/rwa`.

## Testing API Connectivity

1. After starting the application, navigate to the homepage
2. Toggle the "显示API调试工具" switch at the top of the page
3. Use the API Test Component to check that the connection to the backend is working properly

## Authentication Flow

This application implements Web3 authentication with the following flow:

1. User connects their wallet (MetaMask, OKX, or others)
2. Application requests a challenge from the backend `/rwa/user/challenge` endpoint
3. User signs the challenge with their wallet
4. Application sends the signature to the backend `/rwa/user/login` endpoint
5. If valid, the backend returns a JWT token
6. Application stores the token and uses it for all subsequent authenticated requests

## Troubleshooting

If you encounter API connectivity issues:

1. Make sure the backend is running at http://127.0.0.1:8888
2. Check browser console for any error messages
3. Verify the API endpoint paths in the request URLs
4. Ensure setupProxy.js is correctly configured
5. Try restarting the development server
