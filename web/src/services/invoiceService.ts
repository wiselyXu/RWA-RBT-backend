import ApiService from './apiService';

/**
 * 票据结构 (与后端对齐)
 */
export interface Invoice {
  id: string;
  invoice_number: string;
  creditor_id: string; // 后端原始字段
  debtor_id: string;   // 后端原始字段
  amount: number;
  currency: string;
  due_date: string; // ISO 8601 string
  status: string; // e.g., "Pending", "Verified", "Repaid"
  ipfs_hash?: string;
  batch_id?: string; // 对应后端的 batch_id 或 token_batch
  created_at: string; // ISO 8601 string
  updated_at: string; // ISO 8601 string
  payee?: string; // 可能来自 creditor_id 或单独提供
  payer?: string; // 可能来自 debtor_id 或单独提供
  contract_hash?: string;
  blockchain_timestamp?: string;
  token_batch?: string; // 确认此字段是否与 batch_id 重复或用途不同
  is_cleared?: boolean;
  is_valid?: boolean;
  annual_interest_rate: number;
}

/**
 * 创建票据参数 (与后端 DTO 对齐)
 */
export interface CreateInvoiceParams {
  invoiceNumber?: string;
  payee: string;
  payer: string;
  amount: string; // Backend expects U256 string representation
  currency: string; // Added currency
  dueDate: string; // Backend expects Unix timestamp string
  ipfsHash: string;
  contractHash: string;
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
      // 假设后端直接返回票据数组, 如果有包装器 (如 { data: [...] }), 需要相应调整
      const invoices = await this.apiService.get<Invoice[]>('/invoice/list');
      // return response.data || []; // 如果有包装器，则使用此行
      return invoices || [];
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
      // Payload matches the CreateInvoiceParams interface
      // Keys are already camelCase due to the interface definition
      const payload: Partial<CreateInvoiceParams> = {
        payee: invoiceData.payee,
        payer: invoiceData.payer,
        amount: invoiceData.amount,
        currency: invoiceData.currency,
        dueDate: invoiceData.dueDate,
        ipfsHash: invoiceData.ipfsHash,
        contractHash: invoiceData.contractHash,
      };
      // Conditionally add invoiceNumber if it exists (though we won't send it from the dialog)
      if (invoiceData.invoiceNumber) {
        payload.invoiceNumber = invoiceData.invoiceNumber;
      }
      
      // The actual request will send camelCase keys because of the payload object's structure
      const response = await this.apiService.post<Invoice>('/invoice/create', payload);
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
      return response; // Assuming backend returns the invoice object directly
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
      // Assuming the delete endpoint returns a success status or maybe a confirmation message
      // Adjust the return type if the backend returns something specific
      await this.apiService.delete<{ message: string } | void>(`/invoice/del?id=${invoiceId}`);
    } catch (error) {
      console.error(`Failed to delete invoice ${invoiceId}:`, error);
      throw error;
    }
  }
}

export default InvoiceService; 