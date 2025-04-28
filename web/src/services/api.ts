import { ChallengeResponse, LoginResponse } from '../types';

// Adjust the base URL if your API is running elsewhere
// Assuming a proxy is set up in vite.config.ts for /rwa
const API_BASE_URL = '/rwa'; 

export async function getChallenge(address: string): Promise<ChallengeResponse> {
  const response = await fetch(`${API_BASE_URL}/user/challenge`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ address }),
  });
  if (!response.ok) {
    // Handle basic HTTP errors
    // You might want more sophisticated error handling
    const errorData = await response.json().catch(() => ({})); // Try to parse error response
    throw new Error(errorData?.msg || `HTTP error! status: ${response.status}`);
  }
  return response.json();
}

export async function loginWithSignature(requestId: string, signature: string): Promise<LoginResponse> {
  const response = await fetch(`${API_BASE_URL}/user/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ requestId, signature }),
  });
   if (!response.ok) {
     // Handle basic HTTP errors
     const errorData = await response.json().catch(() => ({})); // Try to parse error response
     throw new Error(errorData?.msg || `HTTP error! status: ${response.status}`);
  }
  return response.json();
}

// Add other API functions as needed 