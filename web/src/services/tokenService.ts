import ApiService from './apiService';

class TokenService {
  private static instance: TokenService;
  private apiService: ApiService;

  // 私有构造函数，使用单例模式
  private constructor() {
    this.apiService = ApiService.getInstance();
  }

  // 获取单例实例
  public static getInstance(): TokenService {
    if (!TokenService.instance) {
      TokenService.instance = new TokenService();
    }
    return TokenService.instance;
  }

  /**
   * 创建新的token批次
   * @param batchData token批次参数
   */
  public async createTokenBatch(batchData: {
    batch_reference: string;
    invoice_id: string;
    creditor_id: string;
    debtor_id: string;
    stablecoin_symbol: string;
    total_token_supply: string;
    token_value: string;
    interest_rate_apy: string;
    maturity_date: string;
  }): Promise<any> {
    try {
      const response = await this.apiService.post<any>('/token/create', batchData);
      return response;
    } catch (error) {
      console.error('创建token批次失败:', error);
      throw error;
    }
  }

  /**
   * 获取token批次列表
   * @param filters 过滤条件
   */
  public async getTokenBatches(filters?: {
    status?: string;
    creditor_id?: string;
    stablecoin_symbol?: string;
    page?: number;
    page_size?: number;
  }): Promise<any> {
    try {
      let queryParams = '';
      if (filters) {
        const params = new URLSearchParams();
        if (filters.status) params.append('status', filters.status);
        if (filters.creditor_id) params.append('creditor_id', filters.creditor_id);
        if (filters.stablecoin_symbol) params.append('stablecoin_symbol', filters.stablecoin_symbol);
        if (filters.page) params.append('page', filters.page.toString());
        if (filters.page_size) params.append('page_size', filters.page_size.toString());
        queryParams = `?${params.toString()}`;
      }
      
      const response = await this.apiService.get<any>(`/token/batches${queryParams}`);
      return response;
    } catch (error) {
      console.error('获取token批次列表失败:', error);
      throw error;
    }
  }

  /**
   * 获取token市场列表
   * @param filters 过滤条件
   */
  public async getTokenMarkets(filters?: {
    stablecoin_symbol?: string;
    page?: number;
    page_size?: number;
  }): Promise<any> {
    try {
      let queryParams = '';
      if (filters) {
        const params = new URLSearchParams();
        if (filters.stablecoin_symbol) params.append('stablecoin_symbol', filters.stablecoin_symbol);
        if (filters.page) params.append('page', filters.page.toString());
        if (filters.page_size) params.append('page_size', filters.page_size.toString());
        queryParams = `?${params.toString()}`;
      }
      
      const response = await this.apiService.get<any>(`/token/markets${queryParams}`);
      return response;
    } catch (error) {
      console.error('获取token市场列表失败:', error);
      throw error;
    }
  }

  /**
   * 购买token
   * @param purchaseData 购买参数
   */
  public async purchaseTokens(purchaseData: {
    batch_id: string;
    token_amount: string;
  }): Promise<any> {
    try {
      const response = await this.apiService.post<any>('/token/purchase', purchaseData);
      return response;
    } catch (error) {
      console.error('购买token失败:', error);
      throw error;
    }
  }

  /**
   * 获取用户持有的token
   */
  public async getUserTokenHoldings(): Promise<any> {
    try {
      const response = await this.apiService.get<any>('/token/holdings');
      return response;
    } catch (error) {
      console.error('获取用户token持有列表失败:', error);
      throw error;
    }
  }

  /**
   * 从发票批次创建token批次
   * @param invoiceBatchId 发票批次ID
   * @param batchParams token批次参数
   */
  public async createTokenBatchFromInvoiceBatch(
    invoiceBatchId: string, 
    batchParams: {
      batch_reference: string;
      stablecoin_symbol: string;
      token_value: string;
      interest_rate_apy: string;
      maturity_date?: string;
    }
  ): Promise<any> {
    try {
      const response = await this.apiService.post<any>(
        `/token/from_invoice_batch?invoice_batch_id=${invoiceBatchId}`, 
        batchParams
      );
      return response;
    } catch (error) {
      console.error('从发票批次创建token批次失败:', error);
      throw error;
    }
  }
}

export default TokenService; 