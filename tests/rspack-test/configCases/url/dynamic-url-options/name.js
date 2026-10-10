let calls = 0;

export function getName() {
  calls++;
  return 'dynamic';
}

export function getCalls() {
  return calls;
}
