import { Request, Response } from 'express';
import { metricsAggregationService } from '../services/metricsAggregationService';
import LendingPerformance from '../models/LendingPerformance';

export class MetricsController {
  
  /**
   * GET /admin/metrics/overview
   * Get high-level platform metrics for admin dashboard
   */
  async getOverview(req: Request, res: Response): Promise<Response> {
    try {
      const { timeframe = 'daily' } = req.query;
      
      const overview = await metricsAggregationService.getMetricsOverview(timeframe as string);
      
      return res.status(200).json({
        success: true,
        data: overview
      });
    } catch (error) {
      console.error('Error fetching metrics overview:', error);
      return res.status(500).json({
        success: false,
        message: 'Failed to fetch metrics overview'
      });
    }
  }

  /**
   * GET /admin/metrics/trends
   * Get lending performance trends
   */
  async getTrends(req: Request, res: Response): Promise<Response> {
    try {
      const { days = '30' } = req.query;
      const trends = await metricsAggregationService.getPerformanceTrends(parseInt(days as string));
      
      return res.status(200).json({
        success: true,
        data: trends
      });
    } catch (error) {
      console.error('Error fetching trends:', error);
      return res.status(500).json({
        success: false,
        message: 'Failed to fetch trends'
      });
    }
  }

  /**
   * POST /admin/metrics/aggregate
   * Trigger manual metrics aggregation (for cron jobs)
   */
  async triggerAggregation(req: Request, res: Response): Promise<Response> {
    try {
      const { startDate, endDate } = req.body;
      const start = startDate ? new Date(startDate) : new Date(Date.now() - 24 * 60 * 60 * 1000);
      const end = endDate ? new Date(endDate) : new Date();
      
      await metricsAggregationService.storeMetrics(start, end);
      
      return res.status(200).json({
        success: true,
        message: `Metrics aggregated from ${start.toISOString()} to ${end.toISOString()}`
      });
    } catch (error) {
      console.error('Error aggregating metrics:', error);
      return res.status(500).json({
        success: false,
        message: 'Failed to aggregate metrics'
      });
    }
  }

  /**
   * GET /admin/metrics/dashboard
   * Get dashboard-specific metrics with comparisons
   */
  async getDashboardMetrics(req: Request, res: Response): Promise<Response> {
    try {
      // Get current period metrics (last 30 days)
      const currentPeriod = await metricsAggregationService.getPerformanceTrends(30);
      
      // Get previous period metrics (days 30-60)
      const previousPeriod = await metricsAggregationService.getPerformanceTrends(60);
      
      // Calculate percentage changes
      const currentVolume = currentPeriod.data.reduce((sum, d) => sum + d.volume, 0);
      const previousVolume = previousPeriod.data.slice(0, 30).reduce((sum, d) => sum + d.volume, 0);
      
      const volumeChange = previousVolume > 0 
        ? ((currentVolume - previousVolume) / previousVolume) * 100 
        : 0;
      
      return res.status(200).json({
        success: true,
        data: {
          currentPeriod: currentPeriod,
          previousPeriod: previousPeriod,
          comparisons: {
            volumeChange: volumeChange.toFixed(2),
            trend: volumeChange >= 0 ? 'up' : 'down'
          }
        }
      });
    } catch (error) {
      console.error('Error fetching dashboard metrics:', error);
      return res.status(500).json({
        success: false,
        message: 'Failed to fetch dashboard metrics'
      });
    }
  }
}

export const metricsController = new MetricsController();
