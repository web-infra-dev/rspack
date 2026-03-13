import { readFileSync } from 'node:fs';

export function getServerOnlyInfo() {
  return typeof readFileSync === 'function' ? 'server-only-ok' : 'server-only-missing';
}
