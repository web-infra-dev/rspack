#!/usr/bin/env node

import os from 'os'

export function hello() {
  return 'hello from rslib hashbang' + os.platform();
}
