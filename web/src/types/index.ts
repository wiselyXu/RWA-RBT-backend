// Define backend response types
export interface ChallengeResponse {
  code: number;
  msg: string;
  data: {
    nonce: string;
    requestId: string;
  } | null;
}

export interface LoginResponse {
  code: number;
  msg: string;
  data: {
    message: string;
    token: string;
  } | null;
}

// Add other types as needed 