#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_determine_health_status() {
        let engine = ReserveHealthEngine {
            db: PgPool::connect_lazy("postgres://test").unwrap(),
            min_coverage_ratio: Decimal::new(10, 2),
            warning_coverage_ratio: Decimal::new(15, 2),
            critical_coverage_ratio: Decimal::new(5, 2),
        };

        // Critical status
        let status = engine.determine_health_status(Decimal::new(3, 2), Decimal::from(50));
        assert_eq!(status, "critical");

        // Warning status
        let status = engine.determine_health_status(Decimal::new(12, 2), Decimal::from(50));
        assert_eq!(status, "warning");

        // Healthy status
        let status = engine.determine_health_status(Decimal::new(20, 2), Decimal::from(50));
        assert_eq!(status, "healthy");

        // High utilization
        let status = engine.determine_health_status(Decimal::new(20, 2), Decimal::from(95));
        assert_eq!(status, "high_utilization");
    }

    #[test]
    fn test_coverage_ratio_calculation() {
        let bad_debt_reserve = Decimal::from(100000);
        let utilized = Decimal::from(1000000);
        
        let coverage_ratio = bad_debt_reserve / utilized;
        assert_eq!(coverage_ratio, Decimal::new(10, 2)); // 0.10 = 10%
    }

    #[test]
    fn test_utilization_rate_calculation() {
        let utilized = Decimal::from(750000);
        let total = Decimal::from(1000000);
        
        let utilization_rate = (utilized / total) * Decimal::from(100);
        assert_eq!(utilization_rate, Decimal::from(75)); // 75%
    }
}
