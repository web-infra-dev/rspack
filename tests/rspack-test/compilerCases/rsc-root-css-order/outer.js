'use client';

import { createElement } from 'react';
import { showInner } from './show-inner.js';

export function Outer() {
  return createElement('button', { onClick: showInner }, 'Show inner');
}
