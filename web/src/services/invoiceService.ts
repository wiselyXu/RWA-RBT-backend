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
  due_date: number; // 改为数值型时间戳
  status: string; // e.g., "Pending", "Verified", "Repaid"
  
  // 更新IPFS字段名称
  invoice_ipfs_hash?: string; // 票据IPFS地址
  contract_ipfs_hash?: string; // 合同IPFS地址
  
  // 兼容旧字段
  ipfs_hash?: string; // 兼容旧版
  contract_hash?: string; // 兼容旧版
  
  batch_id?: string; // 对应后端的 batch_id 或 token_batch
  created_at: string; // ISO 8601 string
  updated_at: string; // ISO 8601 string
  payee?: string; // 债权人地址
  payer?: string; // 债务人地址
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
  payee: string;
  payer: string;
  amount: number; // Backend expects U256 string representation
  currency: string; // Added currency
  due_date: number; // Backend expects Unix timestamp string
  invoice_ipfs_hash: string;
  contract_ipfs_hash: string;
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
      console.log('InvoiceService接收到的参数:', invoiceData);
      
      // 直接使用数值类型，不转换为字符串
      const payload = {
        payee: invoiceData.payee,
        payer: invoiceData.payer,
        amount: invoiceData.amount, // 保持为数值类型
        currency: invoiceData.currency,
        due_date: invoiceData.due_date, // 保持为数值类型
        invoice_ipfs_hash: invoiceData.invoice_ipfs_hash,
        contract_ipfs_hash: invoiceData.contract_ipfs_hash,
      };
      
      console.log('发送到后端的最终payload:', payload);
      
      // 发送请求到后端
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