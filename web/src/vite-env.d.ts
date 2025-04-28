/// <reference types="vite/client" />

// Extend the Window interface to include ethereum
interface Window {
  ethereum?: import('ethers').Eip1193Provider; // Use Eip1193Provider type from ethers
}
