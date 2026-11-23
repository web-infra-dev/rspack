'use client';

import { createElement } from 'react';
import { showMiddle } from './show-middle.js';

export function Outer() {
  return createElement('button', { onClick: showMiddle }, 'show middle');
}
