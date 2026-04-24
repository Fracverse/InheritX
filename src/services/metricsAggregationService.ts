import LendingPerformance from '../models/LendingPerformance';
import { Loan, User, Transaction } from '../models'; // Adjust imports based on your models

interface AggregationResult {
  date: Date;
  totalLendingVolume: number;
  activeLoans: number;
  completedLoans: number;
  defaultedLoans: number;
  averageLoanSize: number;
  averageInterestRate: number;
  totalInterestEarned: number;
  uniqueBorrowers: number;
  uniqueLenders: number;
  repaymentRate: number;
}

export class MetricsAggregationService {
  
  /**
   * Aggregate daily lending metrics
   */
  async aggregateDailyMetrics(date: Date): Promise<AggregationResult> {
    const startOfDay = new Date(date);
    startOfDay.setHours(0, 0, 0, 0);
    
    const endOfDay = new Date(date);
    endOfDay.setHours(23, 59, 59, 999);

    // Aggregate loans for the day
    const loans = await Loan.aggregate([
      {
        $match: {
          createdAt: { $gte: startOfDay, $lte: endOfDay }
        }
      },
      {
        $group: {
          _id: null,
          totalVolume: { $sum: '$amount' },
          activeCount: {
            $sum: { $cond: [{ $eq: ['$status', 'active'] }, 1, 0] }
          },
          completedCount: {
            $sum: { $cond: [{ $eq: ['$status', 'completed'] }, 1, 0] }
          },
          defaultedCount: {
            $sum: { $cond: [{ $eq: ['$status', 'defaulted'] }, 1, 0] }
          },
          avgLoanSize: { $avg: '$amount' },
          avgInterestRate: { $avg: '$interestRate' },
          totalInterest: { $sum: '$interestEarned' }
        }
      }
    ]);

    // Get unique borrowers and lenders
    const [borrowers, lenders] = await Promise.all([
      Loan.distinct('borrowerId', { createdAt: { $gte: startOfDay, $lte: endOfDay } }),
      Loan.distinct('lenderId', { createdAt: { $gte: startOfDay, $lte: endOfDay } })
    ]);

    const result = loans[0] || {};
    const totalLoans = (result.activeCount || 0) + (result.completedCount || 0) + (result.defaultedCount || 0);
    const repaymentRate = totalLoans > 0 
      ? ((result.completedCount || 0) / totalLoans) * 100 
      : 0;

    return {
      date: startOfDay,
      totalLendingVolume: result.totalVolume || 0,
      activeLoans: result.activeCount || 0,
      completedLoans: result.completedCount || 0,
      defaultedLoans: result.defaultedCount || 0,
      averageLoanSize: result.avgLoanSize || 0,
      averageInterestRate: result.avgInterestRate || 0,
      totalInterestEarned: result.totalInterest || 0,
      uniqueBorrowers: borrowers.length,
      uniqueLenders: lenders.length,
      repaymentRate
    };
  }

  /**
   * Store aggregated metrics for a date range
   */
  async storeMetrics(startDate: Date, endDate: Date): Promise<void> {
    const currentDate = new Date(startDate);
    
    while (currentDate <= endDate) {
      const metrics = await this.aggregateDailyMetrics(currentDate);
      
      await LendingPerformance.findOneAndUpdate(
        { date: metrics.date },
        metrics,
        { upsert: true, new: true }
      );
      
      currentDate.setDate(currentDate.getDate() + 1);
    }
  }

  /**
   * Get metrics for dashboard/overview
   */
  async getMetricsOverview(timeframe: 'daily' | 'weekly' | 'monthly' = 'daily'): Promise<any> {
    const matchStage: any = {};
    
    if (timeframe === 'weekly') {
      matchStage.date = { $gte: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000) };
    } else if (timeframe === 'monthly') {
      matchStage.date = { $gte: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000) };
    }

    const metrics = await LendingPerformance.aggregate([
      { $match: matchStage },
      {
        $group: {
          _id: timeframe === 'daily' ? '$date' : 
               timeframe === 'weekly' ? { $week: '$date' } : 
               { $month: '$date' },
          totalVolume: { $sum: '$totalLendingVolume' },
          avgLoanSize: { $avg: '$averageLoanSize' },
          avgInterestRate: { $avg: '$averageInterestRate' },
          totalInterest: { $sum: '$totalInterestEarned' },
          avgRepaymentRate: { $avg: '$repaymentRate' },
          totalActiveLoans: { $sum: '$activeLoans' },
          totalCompletedLoans: { $sum: '$completedLoans' },
          totalDefaultedLoans: { $sum: '$defaultedLoans' }
        }
      },
      { $sort: { _id: 1 } }
    ]);

    // Get cumulative totals
    const totals = await LendingPerformance.aggregate([
      {
        $group: {
          _id: null,
          totalLendingVolume: { $sum: '$totalLendingVolume' },
          totalInterestEarned: { $sum: '$totalInterestEarned' },
          averageRepaymentRate: { $avg: '$repaymentRate' }
        }
      }
    ]);

    return {
      metrics,
      totals: totals[0] || {},
      timeframe
    };
  }

  /**
   * Get lending performance trends
   */
  async getPerformanceTrends(period: number = 30): Promise<any> {
    const startDate = new Date(Date.now() - period * 24 * 60 * 60 * 1000);
    
    const trends = await LendingPerformance.find({
      date: { $gte: startDate }
    }).sort({ date: 1 });

    return {
      period: `${period} days`,
      data: trends.map(t => ({
        date: t.date,
        volume: t.totalLendingVolume,
        activeLoans: t.activeLoans,
        repaymentRate: t.repaymentRate,
        interestEarned: t.totalInterestEarned
      }))
    };
  }
}

export const metricsAggregationService = new MetricsAggregationService();
