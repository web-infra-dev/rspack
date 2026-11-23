'use server';

import { createElement } from 'react';
import { Middle } from './middle.js';

export async function showMiddle() {
  return { step: 'middle', element: createElement(Middle) };
}
