// k6 load test: concurrent chat completions
// Run: k6 run tests/load/chat-load.js --env BASE_URL=http://localhost:18789

import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 10 },
    { duration: '1m', target: 50 },
    { duration: '30s', target: 100 },
    { duration: '1m', target: 100 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<5000'],
    http_req_failed: ['rate<0.1'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:18789';

export default function () {
  const payload = JSON.stringify({
    messages: [{ role: 'user', content: `Hello, the time is ${Date.now()}` }],
    stream: false,
  });

  const params = {
    headers: { 'Content-Type': 'application/json' },
  };

  const res = http.post(`${BASE_URL}/api/v1/chat`, payload, params);
  check(res, {
    'status is 200': (r) => r.status === 200,
    'has choices': (r) => JSON.parse(r.body).choices !== undefined,
  });

  sleep(0.5);
}
