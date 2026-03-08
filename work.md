 cargo run --release -- --target $TARGET --port $PORT --host $HOST --insecure --path "/api" --workers 50 --requests 5000
    Finished `release` profile [optimized] target(s) in 0.02s
     Running `target/release/vex --target 127.0.0.1 --port 9889 --host proxy.spooky.local --insecure --path /api --workers 50 --requests 5000`
Starting HTTP/3 load test:
  Target: 127.0.0.1:9889
  Host: proxy.spooky.local
  Path: /api
  Workers: 50
  Total requests: 5000
  Duration: 30s
  Insecure: true

Worker 19: Request 3 failed: timeout waiting for response
Worker 32: Request 3 failed: timeout waiting for response
Worker 9: Request 4 failed: timeout waiting for response
Worker 47: Request 6 failed: timeout waiting for response
Worker 3: Request 5 failed: timeout waiting for response
Worker 15: Request 2 failed: timeout waiting for response
Worker 39: Request 4 failed: timeout waiting for response
Worker 6: Request 5 failed: timeout waiting for response
Worker 10: Request 8 failed: timeout waiting for response
Worker 24: Request 7 failed: timeout waiting for response
Worker 40: Request 9 failed: timeout waiting for response
Worker 5: Request 3 failed: timeout waiting for response
Worker 28: Request 14 failed: timeout waiting for response
Worker 36: Request 9 failed: timeout waiting for response
Worker 33: Request 18 failed: timeout waiting for response
Worker 19: Request 8 failed: timeout waiting for response
Worker 30: Request 12 failed: timeout waiting for response
Worker 17: Request 10 failed: timeout waiting for response
Worker 24: Request 8 failed: timeout waiting for response
Worker 28: Request 15 failed: timeout waiting for response
Worker 39: Request 9 failed: timeout waiting for response
Worker 16: Request 17 failed: timeout waiting for response
Worker 26: Request 17 failed: timeout waiting for response
Worker 47: Request 15 failed: timeout waiting for response
Worker 19: Request 12 failed: timeout waiting for response
Worker 41: Request 19 failed: timeout waiting for response
Worker 4: Request 29 failed: timeout waiting for response
Worker 6: Request 14 failed: timeout waiting for response
Worker 32: Request 16 failed: timeout waiting for response
Worker 20: Request 15 failed: timeout waiting for response
Worker 8: Request 31 failed: timeout waiting for response
Worker 14: Request 23 failed: timeout waiting for response
Worker 39: Request 12 failed: timeout waiting for response
Worker 33: Request 23 failed: timeout waiting for response
Worker 17: Request 15 failed: timeout waiting for response
Worker 0: Request 23 failed: timeout waiting for response
Worker 13: Request 29 failed: timeout waiting for response
Worker 29: Request 19 failed: timeout waiting for response
Worker 7: Request 22 failed: timeout waiting for response
Worker 47: Request 16 failed: timeout waiting for response
Worker 6: Request 16 failed: timeout waiting for response
Worker 1: Request 30 failed: timeout waiting for response
Worker 27: Request 18 failed: timeout waiting for response
Worker 10: Request 12 failed: timeout waiting for response
Worker 34: Request 22 failed: timeout waiting for response
Worker 37: Request 25 failed: timeout waiting for response
Worker 14: Request 28 failed: timeout waiting for response
Worker 25: Request 33 failed: timeout waiting for response
Worker 48: Request 26 failed: timeout waiting for response
Worker 43: Request 22 failed: timeout waiting for response

Load test completed:
  Total time: 34.14s
  Total requests: 1347
  Successful requests: 1297
  Failed requests: 50
  Requests/sec: 39.45
  Completion reason: Duration limit (30s) reached

HTTP Status code breakdown:
  200: 2xx Success (1297)

Latency metrics (ms):
  Min:  83.56
  Max:  4884.61
  Avg:  952.02
  p50:  715.92
  p90:  1849.40
  p95:  3102.87
  p99:  4133.69
nishant@Black-Pearl:~/Documents/rust/vex$ cargo run --release -- --target $TARGET --port $PORT --host $HOST --insecure --path "/api" --workers 100 --requests 10000
    Finished `release` profile [optimized] target(s) in 0.04s
     Running `target/release/vex --target 127.0.0.1 --port 9889 --host proxy.spooky.local --insecure --path /api --workers 100 --requests 10000`
Starting HTTP/3 load test:
  Target: 127.0.0.1:9889
  Host: proxy.spooky.local
  Path: /api
  Workers: 100
  Total requests: 10000
  Duration: 30s
  Insecure: true

Worker 97: Request 0 failed: timeout waiting for response
Worker 64: Request 0 failed: timeout waiting for response
Worker 98: Request 0 failed: timeout waiting for response
Worker 99: Request 0 failed: timeout waiting for response
Worker 68: Request 0 failed: timeout waiting for response
Worker 38: Request 2 failed: timeout waiting for response
Worker 12: Request 3 failed: timeout waiting for response
Worker 83: Request 1 failed: timeout waiting for response
Worker 79: Request 3 failed: timeout waiting for response
Worker 25: Request 9 failed: timeout waiting for response
Worker 85: Request 2 failed: timeout waiting for response
Worker 66: Request 3 failed: timeout waiting for response
Worker 20: Request 11 failed: timeout waiting for response
Worker 90: Request 1 failed: timeout waiting for response
Worker 51: Request 3 failed: timeout waiting for response
Worker 28: Request 5 failed: timeout waiting for response
Worker 32: Request 0 failed: timeout waiting for response
Worker 41: Request 2 failed: timeout waiting for response
Worker 39: Request 6 failed: timeout waiting for response
Worker 43: Request 0 failed: timeout waiting for response
Worker 6: Request 5 failed: timeout waiting for response
Worker 18: Request 11 failed: timeout waiting for response
Worker 57: Request 2 failed: timeout waiting for response
Worker 94: Request 3 failed: timeout waiting for response
Worker 89: Request 1 failed: timeout waiting for response
Worker 35: Request 4 failed: timeout waiting for response
Worker 36: Request 6 failed: timeout waiting for response
Worker 93: Request 3 failed: timeout waiting for response
Worker 14: Request 2 failed: timeout waiting for response
Worker 19: Request 4 failed: timeout waiting for response
Worker 2: Request 5 failed: timeout waiting for response
Worker 95: Request 5 failed: timeout waiting for response
Worker 87: Request 5 failed: timeout waiting for response
Worker 92: Request 6 failed: timeout waiting for response
Worker 78: Request 8 failed: timeout waiting for response
Worker 42: Request 8 failed: timeout waiting for response
Worker 44: Request 9 failed: timeout waiting for response
Worker 9: Request 5 failed: timeout waiting for response
Worker 56: Request 11 failed: timeout waiting for response
Worker 58: Request 4 failed: timeout waiting for response
Worker 88: Request 3 failed: timeout waiting for response
Worker 24: Request 13 failed: timeout waiting for response
Worker 47: Request 5 failed: timeout waiting for response
Worker 69: Request 3 failed: timeout waiting for response
Worker 76: Request 11 failed: timeout waiting for response
Worker 17: Request 6 failed: timeout waiting for response
Worker 79: Request 4 failed: timeout waiting for response
Worker 25: Request 10 failed: timeout waiting for response
Worker 83: Request 2 failed: timeout waiting for response
Worker 97: Request 5 failed: timeout waiting for response
Worker 46: Request 5 failed: timeout waiting for response
Worker 86: Request 9 failed: timeout waiting for response
Worker 22: Request 2 failed: timeout waiting for response
Worker 5: Request 11 failed: timeout waiting for response
Worker 3: Request 10 failed: timeout waiting for response
Worker 1: Request 3 failed: timeout waiting for response
Worker 0: Request 3 failed: timeout waiting for response
Worker 16: Request 8 failed: timeout waiting for response
Worker 66: Request 4 failed: timeout waiting for response
Worker 7: Request 9 failed: timeout waiting for response
Worker 55: Request 8 failed: timeout waiting for response
Worker 65: Request 11 failed: timeout waiting for response
Worker 10: Request 5 failed: timeout waiting for response
Worker 61: Request 8 failed: timeout waiting for response
Worker 45: Request 9 failed: timeout waiting for response
Worker 48: Request 6 failed: timeout waiting for response
Worker 12: Request 6 failed: timeout waiting for response
Worker 43: Request 4 failed: timeout waiting for response
Worker 13: Request 13 failed: timeout waiting for response
Worker 95: Request 6 failed: timeout waiting for response
Worker 29: Request 6 failed: timeout waiting for response
Worker 84: Request 12 failed: timeout waiting for response
Worker 53: Request 7 failed: timeout waiting for response
Worker 27: Request 4 failed: timeout waiting for response
Worker 34: Request 5 failed: timeout waiting for response
Worker 49: Request 10 failed: timeout waiting for response
Worker 78: Request 9 failed: timeout waiting for response
Worker 67: Request 7 failed: timeout waiting for response
Worker 51: Request 4 failed: timeout waiting for response
Worker 28: Request 11 failed: timeout waiting for response
Worker 91: Request 8 failed: timeout waiting for response
Worker 11: Request 6 failed: timeout waiting for response
Worker 36: Request 8 failed: timeout waiting for response
Worker 89: Request 3 failed: timeout waiting for response
Worker 81: Request 8 failed: timeout waiting for response
Worker 42: Request 11 failed: timeout waiting for response
Worker 31: Request 6 failed: timeout waiting for response
Worker 75: Request 9 failed: timeout waiting for response
Worker 19: Request 5 failed: timeout waiting for response
Worker 39: Request 11 failed: timeout waiting for response
Worker 64: Request 8 failed: timeout waiting for response
Worker 47: Request 7 failed: timeout waiting for response
Worker 56: Request 13 failed: timeout waiting for response
Worker 90: Request 3 failed: timeout waiting for response
Worker 62: Request 12 failed: timeout waiting for response
Worker 37: Request 14 failed: timeout waiting for response
Worker 8: Request 9 failed: timeout waiting for response
Worker 44: Request 12 failed: timeout waiting for response
Worker 4: Request 6 failed: timeout waiting for response
Worker 22: Request 3 failed: timeout waiting for response
Worker 74: Request 8 failed: timeout waiting for response
Worker 92: Request 9 failed: timeout waiting for response
Worker 15: Request 13 failed: timeout waiting for response
Worker 82: Request 12 failed: timeout waiting for response
Worker 52: Request 15 failed: timeout waiting for response
Worker 71: Request 14 failed: timeout waiting for response
Worker 41: Request 4 failed: timeout waiting for response
Worker 83: Request 5 failed: timeout waiting for response
Worker 30: Request 7 failed: timeout waiting for response
Worker 80: Request 18 failed: timeout waiting for response
Worker 70: Request 13 failed: timeout waiting for response
Worker 35: Request 7 failed: timeout waiting for response
Worker 68: Request 6 failed: timeout waiting for response
Worker 66: Request 5 failed: timeout waiting for response
Worker 77: Request 12 failed: timeout waiting for response
Worker 9: Request 9 failed: timeout waiting for response
Worker 54: Request 18 failed: timeout waiting for response
Worker 32: Request 7 failed: timeout waiting for response
Worker 26: Request 14 failed: timeout waiting for response
Worker 6: Request 9 failed: timeout waiting for response
Worker 65: Request 12 failed: timeout waiting for response
Worker 12: Request 7 failed: timeout waiting for response
Worker 7: Request 11 failed: timeout waiting for response
Worker 45: Request 10 failed: timeout waiting for response
Worker 40: Request 7 failed: timeout waiting for response
Worker 85: Request 8 failed: timeout waiting for response
Worker 20: Request 16 failed: timeout waiting for response
Worker 21: Request 10 failed: timeout waiting for response
Worker 63: Request 14 failed: timeout waiting for response
Worker 58: Request 9 failed: timeout waiting for response
Worker 79: Request 10 failed: timeout waiting for response
Worker 27: Request 6 failed: timeout waiting for response
Worker 46: Request 7 failed: timeout waiting for response
Worker 33: Request 18 failed: timeout waiting for response
Worker 57: Request 5 failed: timeout waiting for response
Worker 72: Request 13 failed: timeout waiting for response
Worker 17: Request 10 failed: timeout waiting for response
Worker 86: Request 11 failed: timeout waiting for response
Worker 73: Request 15 failed: timeout waiting for response
Worker 87: Request 11 failed: timeout waiting for response
Worker 38: Request 10 failed: timeout waiting for response
Worker 55: Request 9 failed: timeout waiting for response
Worker 43: Request 5 failed: timeout waiting for response
Worker 96: Request 15 failed: timeout waiting for response
Worker 14: Request 5 failed: timeout waiting for response
Worker 81: Request 10 failed: timeout waiting for response
Worker 99: Request 11 failed: timeout waiting for response
Worker 22: Request 4 failed: timeout waiting for response
Worker 94: Request 11 failed: timeout waiting for response
Worker 59: Request 11 failed: timeout waiting for response
Worker 70: Request 14 failed: timeout waiting for response
Worker 75: Request 11 failed: timeout waiting for response
Worker 97: Request 8 failed: timeout waiting for response
Worker 42: Request 13 failed: timeout waiting for response
Worker 88: Request 11 failed: timeout waiting for response
Worker 37: Request 15 failed: timeout waiting for response
Worker 35: Request 8 failed: timeout waiting for response
Worker 29: Request 8 failed: timeout waiting for response
Worker 71: Request 15 failed: timeout waiting for response
Worker 49: Request 14 failed: timeout waiting for response
Worker 78: Request 10 failed: timeout waiting for response
Worker 31: Request 7 failed: timeout waiting for response
Worker 47: Request 9 failed: timeout waiting for response
Worker 4: Request 8 failed: timeout waiting for response
Worker 80: Request 20 failed: timeout waiting for response
Worker 10: Request 7 failed: timeout waiting for response
Worker 6: Request 10 failed: timeout waiting for response
Worker 65: Request 13 failed: timeout waiting for response
Worker 12: Request 8 failed: timeout waiting for response
Worker 90: Request 5 failed: timeout waiting for response
Worker 67: Request 13 failed: timeout waiting for response
Worker 34: Request 9 failed: timeout waiting for response
Worker 46: Request 8 failed: timeout waiting for response
Worker 60: Request 25 failed: timeout waiting for response
Worker 69: Request 10 failed: timeout waiting for response
Worker 98: Request 10 failed: timeout waiting for response
Worker 40: Request 9 failed: timeout waiting for response
Worker 58: Request 11 failed: timeout waiting for response
Worker 17: Request 11 failed: timeout waiting for response
Worker 95: Request 10 failed: timeout waiting for response
Worker 0: Request 8 failed: timeout waiting for response
Worker 54: Request 21 failed: timeout waiting for response
Worker 25: Request 15 failed: timeout waiting for response
Worker 56: Request 14 failed: timeout waiting for response
Worker 16: Request 15 failed: timeout waiting for response
Worker 44: Request 14 failed: timeout waiting for response
Worker 27: Request 7 failed: timeout waiting for response
Worker 62: Request 13 failed: timeout waiting for response
Worker 5: Request 14 failed: timeout waiting for response
Worker 52: Request 18 failed: timeout waiting for response
Worker 89: Request 6 failed: timeout waiting for response
Worker 13: Request 19 failed: timeout waiting for response
Worker 93: Request 13 failed: timeout waiting for response
Worker 11: Request 11 failed: timeout waiting for response
Worker 77: Request 14 failed: timeout waiting for response
Worker 45: Request 12 failed: timeout waiting for response
Worker 2: Request 10 failed: timeout waiting for response
Worker 72: Request 14 failed: timeout waiting for response
Worker 74: Request 11 failed: timeout waiting for response
Worker 68: Request 8 failed: timeout waiting for response
Worker 82: Request 15 failed: timeout waiting for response
Worker 84: Request 19 failed: timeout waiting for response
Worker 96: Request 16 failed: timeout waiting for response
Worker 8: Request 12 failed: timeout waiting for response
Worker 19: Request 7 failed: timeout waiting for response
Worker 23: Request 14 failed: timeout waiting for response
Worker 70: Request 15 failed: timeout waiting for response
Worker 42: Request 14 failed: timeout waiting for response
Worker 91: Request 11 failed: timeout waiting for response
Worker 9: Request 11 failed: timeout waiting for response
Worker 85: Request 10 failed: timeout waiting for response
Worker 61: Request 11 failed: timeout waiting for response
Worker 39: Request 15 failed: timeout waiting for response
Worker 88: Request 12 failed: timeout waiting for response
Worker 71: Request 16 failed: timeout waiting for response
Worker 7: Request 13 failed: timeout waiting for response
Worker 51: Request 9 failed: timeout waiting for response
Worker 73: Request 17 failed: timeout waiting for response
Worker 97: Request 9 failed: timeout waiting for response
Worker 37: Request 16 failed: timeout waiting for response
Worker 31: Request 8 failed: timeout waiting for response
Worker 53: Request 15 failed: timeout waiting for response
Worker 35: Request 9 failed: timeout waiting for response
Worker 10: Request 8 failed: timeout waiting for response
Worker 59: Request 12 failed: timeout waiting for response
Worker 83: Request 7 failed: timeout waiting for response
Worker 24: Request 19 failed: timeout waiting for response
Worker 76: Request 19 failed: timeout waiting for response
Worker 64: Request 13 failed: timeout waiting for response
Worker 28: Request 13 failed: timeout waiting for response
Worker 80: Request 21 failed: timeout waiting for response
Worker 26: Request 18 failed: timeout waiting for response
Worker 78: Request 11 failed: timeout waiting for response
Worker 20: Request 20 failed: timeout waiting for response
Worker 87: Request 14 failed: timeout waiting for response
Worker 12: Request 9 failed: timeout waiting for response
Worker 49: Request 15 failed: timeout waiting for response
Worker 57: Request 8 failed: timeout waiting for response
Worker 29: Request 9 failed: timeout waiting for response
Worker 21: Request 13 failed: timeout waiting for response
Worker 48: Request 13 failed: timeout waiting for response
Worker 63: Request 19 failed: timeout waiting for response
Worker 86: Request 13 failed: timeout waiting for response
Worker 18: Request 23 failed: timeout waiting for response
Worker 15: Request 18 failed: timeout waiting for response
Worker 79: Request 13 failed: timeout waiting for response
Worker 69: Request 11 failed: timeout waiting for response
Worker 95: Request 11 failed: timeout waiting for response
Worker 40: Request 10 failed: timeout waiting for response
Worker 93: Request 14 failed: timeout waiting for response
Worker 43: Request 6 failed: timeout waiting for response
Worker 30: Request 14 failed: timeout waiting for response
Worker 99: Request 13 failed: timeout waiting for response
Worker 32: Request 10 failed: timeout waiting for response
Worker 38: Request 13 failed: timeout waiting for response
Worker 14: Request 6 failed: timeout waiting for response
Worker 96: Request 17 failed: timeout waiting for response
Worker 47: Request 11 failed: timeout waiting for response
Worker 6: Request 12 failed: timeout waiting for response
Worker 8: Request 13 failed: timeout waiting for response
Worker 1: Request 11 failed: timeout waiting for response
Worker 23: Request 15 failed: timeout waiting for response
Worker 67: Request 14 failed: timeout waiting for response
Worker 62: Request 14 failed: timeout waiting for response
Worker 16: Request 16 failed: timeout waiting for response
Worker 5: Request 15 failed: timeout waiting for response
Worker 52: Request 19 failed: timeout waiting for response
Worker 65: Request 15 failed: timeout waiting for response
Worker 89: Request 7 failed: timeout waiting for response
Worker 60: Request 27 failed: timeout waiting for response
Worker 13: Request 20 failed: timeout waiting for response
Worker 36: Request 14 failed: timeout waiting for response
Worker 72: Request 15 failed: timeout waiting for response
Worker 74: Request 12 failed: timeout waiting for response

Load test completed:
  Total time: 36.75s
  Total requests: 1439
  Successful requests: 1165
  Failed requests: 274
  Requests/sec: 39.16
  Completion reason: Duration limit (30s) reached

HTTP Status code breakdown:
  200: 2xx Success (1165)

Latency metrics (ms):
  Min:  109.68
  Max:  5489.85
  Avg:  1268.36
  p50:  865.51
  p90:  3130.22
  p95:  3843.00
  p99:  4554.07
nishant@Black-Pearl:~/Documents/rust/vex$ cargo run --release -- --target $TARGET --port $PORT --host $HOST --insecure --path "/api" --workers 100 --requests 100000
    Finished `release` profile [optimized] target(s) in 0.02s
     Running `target/release/vex --target 127.0.0.1 --port 9889 --host proxy.spooky.local --insecure --path /api --workers 100 --requests 100000`
Starting HTTP/3 load test:
  Target: 127.0.0.1:9889
  Host: proxy.spooky.local
  Path: /api
  Workers: 100
  Total requests: 100000
  Duration: 30s
  Insecure: true

Worker 70: Request 0 failed: timeout waiting for response
Worker 22: Request 0 failed: timeout waiting for response
Worker 58: Request 0 failed: timeout waiting for response
Worker 21: Request 2 failed: timeout waiting for response
Worker 1: Request 2 failed: timeout waiting for response
Worker 91: Request 0 failed: timeout waiting for response
Worker 85: Request 0 failed: timeout waiting for response
Worker 93: Request 0 failed: timeout waiting for response
Worker 66: Request 2 failed: timeout waiting for response
Worker 35: Request 3 failed: timeout waiting for response
Worker 49: Request 0 failed: timeout waiting for response
Worker 31: Request 0 failed: timeout waiting for response
Worker 94: Request 3 failed: timeout waiting for response
Worker 88: Request 3 failed: timeout waiting for response
Worker 84: Request 5 failed: timeout waiting for response
Worker 36: Request 4 failed: timeout waiting for response
Worker 71: Request 1 failed: timeout waiting for response
Worker 39: Request 2 failed: timeout waiting for response
Worker 33: Request 1 failed: timeout waiting for response
Worker 38: Request 6 failed: timeout waiting for response
Worker 78: Request 7 failed: timeout waiting for response
Worker 65: Request 0 failed: timeout waiting for response
Worker 87: Request 1 failed: timeout waiting for response
Worker 82: Request 1 failed: timeout waiting for response
Worker 0: Request 5 failed: timeout waiting for response
Worker 86: Request 2 failed: timeout waiting for response
Worker 50: Request 11 failed: timeout waiting for response
Worker 44: Request 6 failed: timeout waiting for response
Worker 80: Request 6 failed: timeout waiting for response
Worker 12: Request 1 failed: timeout waiting for response
Worker 8: Request 2 failed: timeout waiting for response
Worker 32: Request 10 failed: timeout waiting for response
Worker 4: Request 3 failed: timeout waiting for response
Worker 13: Request 8 failed: timeout waiting for response
Worker 46: Request 5 failed: timeout waiting for response
Worker 5: Request 10 failed: timeout waiting for response
Worker 34: Request 3 failed: timeout waiting for response
Worker 79: Request 5 failed: timeout waiting for response
Worker 19: Request 3 failed: timeout waiting for response
Worker 67: Request 4 failed: timeout waiting for response
Worker 72: Request 2 failed: timeout waiting for response
Worker 76: Request 6 failed: timeout waiting for response
Worker 16: Request 3 failed: timeout waiting for response
Worker 60: Request 10 failed: timeout waiting for response
Worker 26: Request 3 failed: timeout waiting for response
Worker 27: Request 4 failed: timeout waiting for response
Worker 62: Request 3 failed: timeout waiting for response
Worker 21: Request 3 failed: timeout waiting for response
Worker 98: Request 6 failed: timeout waiting for response
Worker 42: Request 5 failed: timeout waiting for response
Worker 55: Request 5 failed: timeout waiting for response
Worker 11: Request 9 failed: timeout waiting for response
Worker 29: Request 8 failed: timeout waiting for response
Worker 73: Request 11 failed: timeout waiting for response
Worker 97: Request 5 failed: timeout waiting for response
Worker 85: Request 2 failed: timeout waiting for response
Worker 66: Request 3 failed: timeout waiting for response
Worker 89: Request 4 failed: timeout waiting for response
Worker 39: Request 3 failed: timeout waiting for response
Worker 68: Request 8 failed: timeout waiting for response
Worker 81: Request 7 failed: timeout waiting for response
Worker 84: Request 6 failed: timeout waiting for response
Worker 69: Request 13 failed: timeout waiting for response
Worker 9: Request 8 failed: timeout waiting for response
Worker 54: Request 9 failed: timeout waiting for response
Worker 40: Request 7 failed: timeout waiting for response
Worker 86: Request 3 failed: timeout waiting for response
Worker 75: Request 13 failed: timeout waiting for response
Worker 95: Request 5 failed: timeout waiting for response
Worker 41: Request 10 failed: timeout waiting for response
Worker 88: Request 5 failed: timeout waiting for response
Worker 93: Request 4 failed: timeout waiting for response
Worker 25: Request 5 failed: timeout waiting for response
Worker 30: Request 3 failed: timeout waiting for response
Worker 71: Request 4 failed: timeout waiting for response
Worker 58: Request 5 failed: timeout waiting for response
Worker 3: Request 3 failed: timeout waiting for response
Worker 77: Request 7 failed: timeout waiting for response
Worker 1: Request 4 failed: timeout waiting for response
Worker 14: Request 12 failed: timeout waiting for response
Worker 8: Request 4 failed: timeout waiting for response
Worker 47: Request 7 failed: timeout waiting for response
Worker 23: Request 9 failed: timeout waiting for response
Worker 0: Request 6 failed: timeout waiting for response
Worker 76: Request 7 failed: timeout waiting for response
Worker 50: Request 12 failed: timeout waiting for response
Worker 43: Request 4 failed: timeout waiting for response
Worker 64: Request 5 failed: timeout waiting for response
Worker 82: Request 2 failed: timeout waiting for response
Worker 15: Request 11 failed: timeout waiting for response
Worker 63: Request 10 failed: timeout waiting for response
Worker 20: Request 5 failed: timeout waiting for response
Worker 79: Request 7 failed: timeout waiting for response
Worker 42: Request 6 failed: timeout waiting for response
Worker 52: Request 12 failed: timeout waiting for response
Worker 91: Request 3 failed: timeout waiting for response
Worker 83: Request 4 failed: timeout waiting for response
Worker 65: Request 3 failed: timeout waiting for response
Worker 2: Request 8 failed: timeout waiting for response
Worker 16: Request 4 failed: timeout waiting for response
Worker 5: Request 12 failed: timeout waiting for response
Worker 62: Request 4 failed: timeout waiting for response
Worker 99: Request 11 failed: timeout waiting for response
Worker 48: Request 8 failed: timeout waiting for response
Worker 67: Request 6 failed: timeout waiting for response
Worker 80: Request 8 failed: timeout waiting for response
Worker 55: Request 7 failed: timeout waiting for response
Worker 73: Request 12 failed: timeout waiting for response
Worker 35: Request 5 failed: timeout waiting for response
Worker 12: Request 5 failed: timeout waiting for response
Worker 37: Request 9 failed: timeout waiting for response
Worker 61: Request 9 failed: timeout waiting for response
Worker 94: Request 7 failed: timeout waiting for response
Worker 24: Request 8 failed: timeout waiting for response
Worker 21: Request 6 failed: timeout waiting for response
Worker 75: Request 14 failed: timeout waiting for response
Worker 87: Request 5 failed: timeout waiting for response
Worker 38: Request 14 failed: timeout waiting for response
Worker 11: Request 10 failed: timeout waiting for response
Worker 6: Request 13 failed: timeout waiting for response
Worker 27: Request 5 failed: timeout waiting for response
Worker 85: Request 5 failed: timeout waiting for response
Worker 70: Request 8 failed: timeout waiting for response
Worker 1: Request 5 failed: timeout waiting for response
Worker 14: Request 13 failed: timeout waiting for response
Worker 9: Request 9 failed: timeout waiting for response
Worker 98: Request 7 failed: timeout waiting for response
Worker 18: Request 12 failed: timeout waiting for response
Worker 78: Request 11 failed: timeout waiting for response
Worker 10: Request 10 failed: timeout waiting for response
Worker 36: Request 10 failed: timeout waiting for response
Worker 97: Request 8 failed: timeout waiting for response
Worker 90: Request 4 failed: timeout waiting for response
Worker 45: Request 10 failed: timeout waiting for response
Worker 32: Request 18 failed: timeout waiting for response
Worker 28: Request 10 failed: timeout waiting for response
Worker 57: Request 5 failed: timeout waiting for response
Worker 58: Request 7 failed: timeout waiting for response
Worker 29: Request 11 failed: timeout waiting for response
Worker 40: Request 9 failed: timeout waiting for response
Worker 92: Request 13 failed: timeout waiting for response
Worker 82: Request 3 failed: timeout waiting for response
Worker 0: Request 7 failed: timeout waiting for response
Worker 64: Request 6 failed: timeout waiting for response
Worker 13: Request 11 failed: timeout waiting for response
Worker 69: Request 17 failed: timeout waiting for response
Worker 65: Request 4 failed: timeout waiting for response
Worker 31: Request 11 failed: timeout waiting for response
Worker 83: Request 5 failed: timeout waiting for response
Worker 63: Request 11 failed: timeout waiting for response
Worker 86: Request 4 failed: timeout waiting for response
Worker 44: Request 11 failed: timeout waiting for response
Worker 52: Request 13 failed: timeout waiting for response
Worker 88: Request 6 failed: timeout waiting for response
Worker 41: Request 12 failed: timeout waiting for response
Worker 95: Request 7 failed: timeout waiting for response
Worker 48: Request 9 failed: timeout waiting for response
Worker 93: Request 8 failed: timeout waiting for response
Worker 47: Request 9 failed: timeout waiting for response
Worker 54: Request 14 failed: timeout waiting for response
Worker 15: Request 13 failed: timeout waiting for response
Worker 72: Request 6 failed: timeout waiting for response
Worker 56: Request 12 failed: timeout waiting for response
Worker 73: Request 13 failed: timeout waiting for response
Worker 4: Request 10 failed: timeout waiting for response
Worker 60: Request 13 failed: timeout waiting for response
Worker 53: Request 15 failed: timeout waiting for response
Worker 77: Request 11 failed: timeout waiting for response
Worker 71: Request 6 failed: timeout waiting for response
Worker 84: Request 10 failed: timeout waiting for response
Worker 20: Request 7 failed: timeout waiting for response
Worker 34: Request 7 failed: timeout waiting for response
Worker 89: Request 10 failed: timeout waiting for response
Worker 74: Request 10 failed: timeout waiting for response
Worker 43: Request 8 failed: timeout waiting for response
Worker 96: Request 12 failed: timeout waiting for response
Worker 38: Request 15 failed: timeout waiting for response
Worker 66: Request 9 failed: timeout waiting for response
Worker 8: Request 6 failed: timeout waiting for response
Worker 59: Request 19 failed: timeout waiting for response
Worker 2: Request 11 failed: timeout waiting for response
Worker 37: Request 10 failed: timeout waiting for response
Worker 76: Request 9 failed: timeout waiting for response
Worker 21: Request 7 failed: timeout waiting for response
Worker 19: Request 10 failed: timeout waiting for response
Worker 75: Request 16 failed: timeout waiting for response
Worker 22: Request 9 failed: timeout waiting for response
Worker 39: Request 9 failed: timeout waiting for response
Worker 14: Request 14 failed: timeout waiting for response
Worker 85: Request 6 failed: timeout waiting for response
Worker 27: Request 6 failed: timeout waiting for response
Worker 57: Request 6 failed: timeout waiting for response
Worker 42: Request 9 failed: timeout waiting for response
Worker 29: Request 12 failed: timeout waiting for response
Worker 40: Request 10 failed: timeout waiting for response
Worker 25: Request 11 failed: timeout waiting for response
Worker 36: Request 11 failed: timeout waiting for response
Worker 9: Request 10 failed: timeout waiting for response
Worker 99: Request 15 failed: timeout waiting for response
Worker 82: Request 4 failed: timeout waiting for response
Worker 26: Request 9 failed: timeout waiting for response
Worker 3: Request 8 failed: timeout waiting for response
Worker 68: Request 12 failed: timeout waiting for response
Worker 78: Request 12 failed: timeout waiting for response
Worker 18: Request 14 failed: timeout waiting for response
Worker 44: Request 12 failed: timeout waiting for response
Worker 12: Request 8 failed: timeout waiting for response
Worker 11: Request 11 failed: timeout waiting for response
Worker 86: Request 5 failed: timeout waiting for response
Worker 7: Request 23 failed: timeout waiting for response
Worker 91: Request 8 failed: timeout waiting for response
Worker 6: Request 16 failed: timeout waiting for response
Worker 48: Request 10 failed: timeout waiting for response
Worker 28: Request 12 failed: timeout waiting for response
Worker 46: Request 11 failed: timeout waiting for response
Worker 49: Request 8 failed: timeout waiting for response
Worker 45: Request 11 failed: timeout waiting for response
Worker 67: Request 9 failed: timeout waiting for response
Worker 47: Request 10 failed: timeout waiting for response
Worker 61: Request 13 failed: timeout waiting for response
Worker 4: Request 11 failed: timeout waiting for response
Worker 52: Request 14 failed: timeout waiting for response
Worker 16: Request 9 failed: timeout waiting for response
Worker 32: Request 19 failed: timeout waiting for response
Worker 84: Request 11 failed: timeout waiting for response
Worker 33: Request 12 failed: timeout waiting for response
Worker 55: Request 9 failed: timeout waiting for response
Worker 65: Request 5 failed: timeout waiting for response
Worker 80: Request 12 failed: timeout waiting for response
Worker 98: Request 10 failed: timeout waiting for response
Worker 89: Request 11 failed: timeout waiting for response
Worker 31: Request 13 failed: timeout waiting for response
Worker 56: Request 13 failed: timeout waiting for response
Worker 70: Request 10 failed: timeout waiting for response
Worker 24: Request 11 failed: timeout waiting for response
Worker 90: Request 7 failed: timeout waiting for response
Worker 60: Request 14 failed: timeout waiting for response
Worker 34: Request 8 failed: timeout waiting for response
Worker 8: Request 7 failed: timeout waiting for response
Worker 38: Request 16 failed: timeout waiting for response
Worker 10: Request 13 failed: timeout waiting for response
Worker 96: Request 13 failed: timeout waiting for response
Worker 74: Request 11 failed: timeout waiting for response
Worker 63: Request 15 failed: timeout waiting for response
Worker 85: Request 7 failed: timeout waiting for response
Worker 79: Request 13 failed: timeout waiting for response
Worker 17: Request 18 failed: timeout waiting for response
Worker 95: Request 9 failed: timeout waiting for response
Worker 94: Request 10 failed: timeout waiting for response
Worker 36: Request 12 failed: timeout waiting for response
Worker 40: Request 11 failed: timeout waiting for response
Worker 21: Request 9 failed: timeout waiting for response
Worker 6: Request 17 failed: timeout waiting for response
Worker 82: Request 5 failed: timeout waiting for response
Worker 26: Request 10 failed: timeout waiting for response
Worker 48: Request 11 failed: timeout waiting for response
Worker 97: Request 12 failed: timeout waiting for response
Worker 3: Request 9 failed: timeout waiting for response
Worker 67: Request 10 failed: timeout waiting for response
Worker 71: Request 9 failed: timeout waiting for response
Worker 86: Request 7 failed: timeout waiting for response
Worker 98: Request 11 failed: timeout waiting for response
Worker 58: Request 10 failed: timeout waiting for response
Worker 2: Request 13 failed: timeout waiting for response
Worker 42: Request 10 failed: timeout waiting for response
Worker 93: Request 10 failed: timeout waiting for response
Worker 50: Request 18 failed: timeout waiting for response
Worker 19: Request 11 failed: timeout waiting for response
Worker 18: Request 15 failed: timeout waiting for response
Worker 44: Request 13 failed: timeout waiting for response
Worker 37: Request 13 failed: timeout waiting for response

Load test completed:
  Total time: 37.13s
  Total requests: 1374
  Successful requests: 1103
  Failed requests: 271
  Requests/sec: 37.01
  Completion reason: Duration limit (30s) reached

HTTP Status code breakdown:
  200: 2xx Success (1103)

Latency metrics (ms):
  Min:  115.48
  Max:  5229.66
  Avg:  1371.99
  p50:  1003.33
  p90:  3518.38
  p95:  4056.94
  p99:  4849.00
