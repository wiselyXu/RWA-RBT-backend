import React from 'react';
import { useWallet } from '../hooks/useWallet'; // Adjust path if needed

// Basic styling (consider moving to a CSS file)
const headerStyle: React.CSSProperties = {
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  padding: '1rem 2rem',
  backgroundColor: '#282c34',
  color: 'white',
  position: 'sticky', // Make header sticky
  top: 0, // Stick to the top
  zIndex: 1000, // Ensure it's above other content
};

const navStyle: React.CSSProperties = {
  display: 'flex',
  gap: '1rem',
};

const buttonStyle: React.CSSProperties = {
  padding: '0.5rem 1rem',
  cursor: 'pointer',
  backgroundColor: '#61dafb',
  border: 'none',
  borderRadius: '4px',
  color: '#282c34',
  fontWeight: 'bold',
};

const disabledButtonStyle: React.CSSProperties = {
    ...buttonStyle,
    cursor: 'not-allowed',
    opacity: 0.6,
};

const addressStyle: React.CSSProperties = {
    backgroundColor: '#444',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    fontFamily: 'monospace',
    marginRight: '1rem', // Add space before disconnect button
};

const errorStyle: React.CSSProperties = {
    color: 'red',
    fontSize: '0.8rem',
    marginTop: '0.5rem',
    textAlign: 'right',
    position: 'absolute',
    top: '4rem', // Position below the header
    right: '2rem',
    maxWidth: '300px' // Prevent error message from being too wide
}

export const Header: React.FC = () => {
  const { address, token, isLoading, error, connectWallet, disconnectWallet } = useWallet();

  // Function to shorten address
  const shortenAddress = (addr: string) => `${addr.substring(0, 6)}...${addr.substring(addr.length - 4)}`;

  return (
    <header style={headerStyle}>
      <div>Pharos RWA Platform</div> {/* Placeholder for Logo/Title */}
      <div style={{ textAlign: 'right'}}> {/* Wrapper for button and error */} 
        <nav style={navStyle}>
          {/* Placeholder Nav Links - Consider using React Router for navigation */}
          <a href="#" style={{ color: 'white' }}>首页</a>
          <a href="#" style={{ color: 'white' }}>帮助中心</a>

          {address && token ? (
              <div style={{display: 'flex', alignItems: 'center'}}>
                  <span style={addressStyle} title={address}>
                      {shortenAddress(address)}
                  </span>
                  <button onClick={disconnectWallet} style={buttonStyle}>
                    断开连接
                  </button>
              </div>
          ) : (
            <button 
              onClick={connectWallet} 
              disabled={isLoading} 
              style={isLoading ? disabledButtonStyle : buttonStyle}
            >
              {isLoading ? '连接中...' : address ? '登录中...' : '连接钱包'} {/* Show intermediate state */} 
            </button>
          )}
        </nav>
        {error && <div style={errorStyle}>Error: {error}</div>} {/* Improved error display */} 
      </div>
    </header>
  );
}; 