import http from 'k6/http';
import { check, sleep } from 'k6';

// Test configuration
export let options = {
  stages: [
    { duration: '2m', target: 100 }, // Ramp up to 100 users over 2 minutes
    { duration: '5m', target: 100 }, // Stay at 100 users for 5 minutes
    { duration: '2m', target: 500 }, // Ramp up to 500 users over 2 minutes
    { duration: '10m', target: 500 }, // Stay at 500 users for 10 minutes
    { duration: '2m', target: 1000 }, // Ramp up to 1000 users over 2 minutes
    { duration: '10m', target: 1000 }, // Stay at 1000 users for 10 minutes
    { duration: '2m', target: 0 }, // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests should be below 500ms
    http_req_failed: ['rate<0.1'], // Error rate should be below 10%
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

// Test scenarios
export default function () {
  // Plan creation scenario
  createPlan();

  // Claim scenario
  claimPlan();

  // Loan operations
  performLoanOperations();

  sleep(1);
}

function createPlan() {
  let payload = JSON.stringify({
    title: 'Load Test Plan',
    description: 'Plan created during load testing',
    beneficiaries: [
      {
        name: 'Test Beneficiary',
        email: 'beneficiary@example.com',
        percentage: 100
      }
    ],
    assets: [
      {
        asset_code: 'USD',
        amount: 10000
      }
    ]
  });

  let params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer ' + getAuthToken(),
    },
  };

  let response = http.post(`${BASE_URL}/api/plans`, payload, params);

  check(response, {
    'plan creation status is 201': (r) => r.status === 201,
    'plan creation response time < 500ms': (r) => r.timings.duration < 500,
  });
}

function claimPlan() {
  // This would require a valid plan ID, simplified for demo
  let response = http.get(`${BASE_URL}/api/plans/due-for-claim`, {
    headers: {
      'Authorization': 'Bearer ' + getAuthToken(),
    },
  });

  check(response, {
    'claim check status is 200': (r) => r.status === 200,
    'claim check response time < 500ms': (r) => r.timings.duration < 500,
  });
}

function performLoanOperations() {
  // Simulate loan creation and operations
  let payload = JSON.stringify({
    amount: 5000,
    asset_code: 'USD',
    duration_months: 12,
    interest_rate: 5.0
  });

  let params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer ' + getAuthToken(),
    },
  };

  let response = http.post(`${BASE_URL}/api/loans/lifecycle`, payload, params);

  check(response, {
    'loan creation status is 201': (r) => r.status === 201,
    'loan creation response time < 500ms': (r) => r.timings.duration < 500,
  });
}

function getAuthToken() {
  // In a real scenario, this would authenticate and return a token
  // For load testing, you might use a pool of pre-authenticated tokens
  return 'test-token';
}