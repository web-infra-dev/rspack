'use server';

import { createElement } from 'react';
import { Inner } from './inner.js';

export async function showInner() {
  return createElement(Inner);
}
