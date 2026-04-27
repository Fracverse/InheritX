# Load Testing Suite

This directory contains load testing scripts for the InheritX backend API.

## Prerequisites

- [k6](https://k6.io/) installed
- Running InheritX backend instance

## Running Load Tests

### Basic Load Test

```bash
k6 run load-test.js
```

### With Custom Base URL

```bash
BASE_URL=http://your-backend-url:8080 k6 run load-test.js
```

### HTML Report

```bash
k6 run --out json=results.json load-test.js
```

## Test Scenarios

The load test includes the following scenarios:

1. **Plan Creation**: Tests creating inheritance plans
2. **Plan Claims**: Tests checking and claiming due plans
3. **Loan Operations**: Tests loan lifecycle operations

## Performance Benchmarks

- **Target**: Handle 500 concurrent users
- **Response Time**: 95th percentile < 500ms
- **Error Rate**: < 10%

## CI/CD Integration

Add to your CI/CD pipeline:

```yaml
- name: Run Load Tests
  run: |
    cd backend/load-testing
    k6 run load-test.js
```

## Monitoring

During load tests, monitor:

- CPU usage
- Memory usage
- Database connections
- Response times
- Error rates