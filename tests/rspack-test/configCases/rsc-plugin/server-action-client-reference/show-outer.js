'use server';

import { createElement } from 'react';
import { Outer } from './outer.js';

export async function showOuter() {
  return { step: 'outer', element: createElement(Outer) };
}
