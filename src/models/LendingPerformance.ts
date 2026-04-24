import { Schema, model, Document } from 'mongoose';
// or use Prisma schema if applicable

export interface ILendingPerformance extends Document {
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
  createdAt: Date;
  updatedAt: Date;
}

const LendingPerformanceSchema = new Schema({
  date: { type: Date, required: true, unique: true },
  totalLendingVolume: { type: Number, default: 0 },
  activeLoans: { type: Number, default: 0 },
  completedLoans: { type: Number, default: 0 },
  defaultedLoans: { type: Number, default: 0 },
  averageLoanSize: { type: Number, default: 0 },
  averageInterestRate: { type: Number, default: 0 },
  totalInterestEarned: { type: Number, default: 0 },
  uniqueBorrowers: { type: Number, default: 0 },
  uniqueLenders: { type: Number, default: 0 },
  repaymentRate: { type: Number, default: 0 },
}, { timestamps: true });

export default model<ILendingPerformance>('LendingPerformance', LendingPerformanceSchema);
