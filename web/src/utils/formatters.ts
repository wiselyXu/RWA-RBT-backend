/**
 * 格式化金额，添加千位分隔符
 * @param amount 金额数值
 * @param decimals 小数位数，默认2位
 * @returns 格式化后的金额字符串
 */
export const formatAmount = (amount: number, decimals: number = 2): string => {
  return amount.toLocaleString('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
};

/**
 * 格式化日期
 * @param date 日期对象或ISO日期字符串
 * @param format 格式，默认为'YYYY-MM-DD'
 * @returns 格式化后的日期字符串
 */
export const formatDate = (date: Date | string): string => {
  const dateObj = typeof date === 'string' ? new Date(date) : date;
  return dateObj.toLocaleDateString();
};

/**
 * 格式化状态，转换为中文显示
 * @param status 状态值
 * @returns 格式化后的状态
 */
export const formatStatus = (status: string): string => {
  const statusMap: Record<string, string> = {
    'Pending': '待处理',
    'Verified': '已验证',
    'Issued': '已发行',
    'Trading': '交易中',
    'Repaying': '还款中',
    'Settled': '已结算',
    'Defaulted': '已违约',
    'Packaging': '打包中',
    'Available': '可购买',
    'Funding': '募资中',
    'Funded': '已募集',
    'Cancelled': '已取消',
    'Completed': '已完成',
    'Expired': '已过期',
  };
  
  return statusMap[status] || status;
}; 