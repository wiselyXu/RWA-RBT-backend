import AuthService from '../services/authService';

/**
 * Intercepts and modifies the native fetch API to add authorization tokens to requests
 */
export const setupHttpInterceptors = () => {
  // Save the original fetch function
  const originalFetch = window.fetch;

  // Override the global fetch function
  window.fetch = async function (input: RequestInfo | URL, init?: RequestInit) {
    // Get the auth service
    const authService = AuthService.getInstance();

    // Only add auth headers if it's an API request (starts with /rwa)
    let authHeaders = {};
    const url = input instanceof URL ? input.toString() : input.toString();
    if (url.includes('/rwa')) {
      authHeaders = authService.getAuthHeader();
    }
    
    // Clone and modify the request init object
    const modifiedInit: RequestInit = {
      ...init,
      headers: {
        ...(init?.headers || {}),
        ...authHeaders,
      },
    };

    // Call the original fetch with modified parameters
    const response = await originalFetch(input, modifiedInit);

    // Handle 401 Unauthorized responses globally
    if (response.status === 401) {
      // If token is invalid/expired, logout the user
      authService.logout();
      
      // Optionally redirect to login page
      if (window.location.pathname !== '/') {
        window.location.href = '/';
      }
    }

    return response;
  };
};

export default setupHttpInterceptors; 