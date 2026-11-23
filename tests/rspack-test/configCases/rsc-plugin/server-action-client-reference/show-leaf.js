'use server';

import { createElement } from 'react';
import { Leaf } from './leaf.js';

export async function showLeaf() {
  return { step: 'leaf', element: createElement(Leaf) };
}
