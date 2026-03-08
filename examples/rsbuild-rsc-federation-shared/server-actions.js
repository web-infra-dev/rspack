'use server';

import { mixedValue } from './mixed-parts.js';
import { sharedValue } from './values.js';

export async function sharedAction() {
  return sharedValue;
}

export async function mixedServerAction(label) {
  return `${sharedValue}:${mixedValue.kind}:${label}`;
}
