'use client';

import { createElement } from 'react';
import { showInner } from './show-inner.js';

export function Middle() {
  return createElement('button', { onClick: showInner }, 'show inner');
}
