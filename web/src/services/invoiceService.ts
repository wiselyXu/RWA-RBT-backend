import ApiService from './apiService';

/**
 * 票据结构
 */
export interface Invoice {
  id: string;
  invoice_number: string;
  creditor_id: string;
  debtor_id: string;
  amount: number;
  currency: string;
  due_date: string;
  status: string;
  ipfs_hash?: string;
  batch_id?: string;
  created_at: string;
  updated_at: string;
  payee?: string;
  payer?: string;
  contract_hash?: string;
  blockchain_timestamp?: string;
  token_batch?: string;
  is_cleared?: boolean;
  is_valid?: boolean;
  annual_interest_rate: number;
}

/**
 * 创建票据参数
 */
export interface CreateInvoiceParams {
  invoice_number: string;
  payee: string;
  payer: string;
  amount: string;
  ipfs_hash: string;
  contract_hash: string;
  timestamp: string;
  due_date: string;
  token_batch: string;
  is_cleared: boolean;
  is_valid: boolean;
}

class InvoiceService {
  private static instance: InvoiceService;
  private apiService: ApiService;

  private constructor() {
    this.apiService = ApiService.getInstance();
  }

  public static getInstance(): InvoiceService {
    if (!InvoiceService.instance) {
      InvoiceService.instance = new InvoiceService();
    }
    return InvoiceService.instance;
  }

  /**
   * 获取当前用户的票据列表
   */
  public async getUserInvoices(): Promise<Invoice[]> {
    try {
      const response = await this.apiService.get<{ data: Invoice[] }>('/invoice/list');
      return response.data || [];
    } catch (error) {
      console.error('Failed to get user invoices:', error);
      throw error;
    }
  }

  /**
   * 创建新票据
   */
  public async createInvoice(invoiceData: CreateInvoiceParams): Promise<Invoice> {
    try {
      const response = await this.apiService.post<Invoice>('/invoice/create', invoiceData);
      return response;
    } catch (error) {
      console.error('Failed to create invoice:', error);
      throw error;
    }
  }

  /**
   * 获取票据详情
   */
  public async getInvoiceDetail(invoiceId: string): Promise<Invoice> {
    try {
      const response = await this.apiService.get<Invoice>(`/invoice/detail?id=${invoiceId}`);
      return response;
    } catch (error) {
      console.error(`Failed to get invoice ${invoiceId} detail:`, error);
      throw error;
    }
  }

  /**
   * 删除票据
   */
  public async deleteInvoice(invoiceId: string): Promise<void> {
    try {
      await this.apiService.delete(`/invoice/del?id=${invoiceId}`);
    } catch (error) {
      console.error(`Failed to delete invoice ${invoiceId}:`, error);
      throw error;
    }
  }
}

export default InvoiceService; 