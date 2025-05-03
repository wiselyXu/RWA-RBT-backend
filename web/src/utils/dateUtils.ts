/**
 * 日期工具函数集
 */

/**
 * 将数值型时间戳（秒）转换为格式化的日期字符串
 * @param timestamp 时间戳（秒）或日期字符串
 * @param format 日期格式，默认为'YYYY-MM-DD'
 * @returns 格式化后的日期字符串
 */
export const formatDate = (timestamp: number | string, format: string = 'YYYY-MM-DD'): string => {
  if (!timestamp) return '-';

  try {
    // 如果输入是数值类型，将其视为Unix时间戳（秒）
    const date = typeof timestamp === 'number' 
      ? new Date(timestamp * 1000) // 转换秒为毫秒
      : new Date(timestamp);

    if (isNaN(date.getTime())) {
      console.warn('Invalid date input:', timestamp);
      return '-';
    }

    // 基本格式化
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    const seconds = String(date.getSeconds()).padStart(2, '0');

    // 根据请求的格式替换
    return format
      .replace('YYYY', String(year))
      .replace('MM', month)
      .replace('DD', day)
      .replace('HH', hours)
      .replace('mm', minutes)
      .replace('ss', seconds);
    
  } catch (error) {
    console.error('Error formatting date:', error);
    return '-';
  }
};

/**
 * 将日期转换为Unix时间戳（秒）
 * @param date Date对象或日期字符串
 * @returns 时间戳（秒）
 */
export const dateToTimestamp = (date: Date | string): number => {
  if (!date) return 0;
  
  try {
    const dateObj = typeof date === 'string' ? new Date(date) : date;
    return Math.floor(dateObj.getTime() / 1000);
  } catch (error) {
    console.error('Error converting date to timestamp:', error);
    return 0;
  }
};

/**
 * 检查日期是否已过期（早于当前日期）
 * @param timestamp 时间戳（秒）或日期字符串
 * @returns 是否已过期
 */
export const isDateExpired = (timestamp: number | string): boolean => {
  if (!timestamp) return false;
  
  try {
    const date = typeof timestamp === 'number' 
      ? new Date(timestamp * 1000)
      : new Date(timestamp);
    
    return date < new Date();
  } catch (error) {
    console.error('Error checking date expiration:', error);
    return false;
  }
}; 