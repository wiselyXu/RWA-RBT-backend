import AuthService from './authService';

// API base URL
const API_BASE_URL = '/rwa';

/**
 * General purpose API service that handles authenticated API requests
 */
class ApiService {
  private static instance: ApiService;
  private authService: AuthService;

  private constructor() {
    this.authService = AuthService.getInstance();
  }

  public static getInstance(): ApiService {
    if (!ApiService.instance) {
      ApiService.instance = new ApiService();
    }
    return ApiService.instance;
  }

  /**
   * Generic GET request with authentication header
   */
  public async get<T>(endpoint: string, params?: Record<string, string>): Promise<T> {
    // Create a URL object for the request
    const url = new URL(`${window.location.origin}${API_BASE_URL}${endpoint}`);
    
    // Add query parameters if provided
    if (params) {
      Object.keys(params).forEach(key => {
        url.searchParams.append(key, params[key]);
      });
    }

    console.log(`Making GET request to: ${url.toString()}`);

    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...this.authService.getAuthHeader(),
    };

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers,
    });

    if (!response.ok) {
      await this.handleErrorResponse(response);
    }

    const data = await response.json();
    return data.data as T;
  }

  /**
   * Generic POST request with authentication header
   */
  public async post<T>(endpoint: string, body: any): Promise<T> {
    // Create the full URL path
    const url = `${API_BASE_URL}${endpoint}`;
    
    console.log(`Making POST request to: ${url} with body:`, body);

    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...this.authService.getAuthHeader(),
    };

    const response = await fetch(url, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      await this.handleErrorResponse(response);
    }

    const data = await response.json();
    return data.data as T;
  }

  /**
   * Generic PUT request with authentication header
   */
  public async put<T>(endpoint: string, body: any): Promise<T> {
    // Create the full URL path
    const url = `${API_BASE_URL}${endpoint}`;
    
    console.log(`Making PUT request to: ${url} with body:`, body);

    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...this.authService.getAuthHeader(),
    };

    const response = await fetch(url, {
      method: 'PUT',
      headers,
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      await this.handleErrorResponse(response);
    }

    const data = await response.json();
    return data.data as T;
  }

  /**
   * Generic DELETE request with authentication header
   */
  public async delete<T>(endpoint: string): Promise<T> {
    // Create the full URL path
    const url = `${API_BASE_URL}${endpoint}`;
    
    console.log(`Making DELETE request to: ${url}`);

    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...this.authService.getAuthHeader(),
    };

    const response = await fetch(url, {
      method: 'DELETE',
      headers,
    });

    if (!response.ok) {
      await this.handleErrorResponse(response);
    }

    const data = await response.json();
    return data.data as T;
  }

  /**
   * Handle error responses and unauthorized status
   */
  private async handleErrorResponse(response: Response): Promise<never> {
    console.error(`API error: ${response.status} ${response.statusText}`);
    
    // If unauthorized (401), log the user out
    if (response.status === 401) {
      this.authService.logout();
      window.location.href = '/'; // Redirect to home/login page
    }

    // Try to parse error message from response
    let errorMessage: string;
    try {
      const errorData = await response.json();
      errorMessage = errorData.message || `Error: ${response.status} ${response.statusText}`;
    } catch {
      errorMessage = `Error: ${response.status} ${response.statusText}`;
    }

    throw new Error(errorMessage);
  }

  /**
   * 获取用户票据列表
   */
  public async getUserInvoices() {
    return this.get('/invoice/list');
  }

  /**
   * 创建新票据
   */
  public async createInvoice(invoiceData: {
    invoice_number: string,
    payee: string,
    payer: string,
    amount: string,
    ipfs_hash: string,
    contract_hash: string,
    timestamp: string,
    due_date: string,
    token_batch: string,
    is_cleared: boolean,
    is_valid: boolean
  }) {
    return this.post('/invoice/create', invoiceData);
  }
}

export default ApiService; 