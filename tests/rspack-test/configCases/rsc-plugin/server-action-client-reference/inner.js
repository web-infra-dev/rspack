'use client';

import { createElement } from 'react';
import { showLeaf } from './show-leaf.js';

export function Inner() {
  return createElement('button', { onClick: showLeaf }, 'show leaf');
}
