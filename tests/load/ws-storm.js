// k6 WebSocket storm test
// Run: k6 run tests/load/ws-storm.js --env BASE_URL=ws://localhost:18789

import ws from 'k6/ws';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '15s', target: 20 },
    { duration: '30s', target: 50 },
    { duration: '15s', target: 0 },
  ],
};

const BASE_URL = __ENV.BASE_URL || 'ws://localhost:18789';

export default function () {
  const url = `${BASE_URL}/ws`;

  const res = ws.connect(url, {}, function (socket) {
    socket.on('open', () => {
      socket.send(
        JSON.stringify({
          type: 'chat',
          payload: {
            messages: [{ role: 'user', content: 'Quick test' }],
          },
        })
      );
    });

    socket.on('message', (msg) => {
      const data = JSON.parse(msg);
      if (data.type === 'done' || data.type === 'error') {
        socket.close();
      }
    });

    socket.setTimeout(() => socket.close(), 10000);
  });

  check(res, { 'WS status is 101': (r) => r && r.status === 101 });
  sleep(1);
}
