'use client';

import { createElement } from 'react';
import { showOuter } from './show-outer.js';

export function Leaf() {
  return createElement('button', { onClick: showOuter }, 'back to outer');
}
