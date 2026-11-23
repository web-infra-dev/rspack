'use client';

import { createElement } from 'react';
import { showInner } from './actions.js';

export function Outer() {
  return createElement('button', { onClick: showInner }, 'show inner');
}
