import { Router } from 'express';
import { metricsController } from '../controllers/metricsController';
import { authenticate, requireAdmin } from '../middleware/auth';

const router = Router();

// All routes require admin authentication
router.use(authenticate);
router.use(requireAdmin);

// GET /admin/metrics/overview - Platform overview metrics
router.get('/overview', metricsController.getOverview.bind(metricsController));

// GET /admin/metrics/trends - Performance trends
router.get('/trends', metricsController.getTrends.bind(metricsController));

// GET /admin/metrics/dashboard - Dashboard metrics with comparisons
router.get('/dashboard', metricsController.getDashboardMetrics.bind(metricsController));

// POST /admin/metrics/aggregate - Manual aggregation trigger
router.post('/aggregate', metricsController.triggerAggregation.bind(metricsController));

export default router;
