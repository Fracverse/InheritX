import cron from 'node-cron';
import { metricsAggregationService } from '../services/metricsAggregationService';

/**
 * Schedule daily metrics aggregation at 01:00 AM
 */
export function scheduleMetricsAggregation(): void {
  // Run daily at 1 AM
  cron.schedule('0 1 * * *', async () => {
    console.log('[MetricsJob] Starting daily metrics aggregation...');
    
    try {
      const yesterday = new Date();
      yesterday.setDate(yesterday.getDate() - 1);
      
      await metricsAggregationService.storeMetrics(yesterday, yesterday);
      
      console.log('[MetricsJob] Daily metrics aggregation completed');
    } catch (error) {
      console.error('[MetricsJob] Failed to aggregate metrics:', error);
    }
  });
  
  // Also run weekly aggregation on Sundays at 2 AM
  cron.schedule('0 2 * * 0', async () => {
    console.log('[MetricsJob] Starting weekly metrics aggregation...');
    
    try {
      const endDate = new Date();
      const startDate = new Date();
      startDate.setDate(startDate.getDate() - 7);
      
      await metricsAggregationService.storeMetrics(startDate, endDate);
      
      console.log('[MetricsJob] Weekly metrics aggregation completed');
    } catch (error) {
      console.error('[MetricsJob] Failed to aggregate weekly metrics:', error);
    }
  });
}
