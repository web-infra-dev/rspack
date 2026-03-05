'use client';

export const sharedValue = 'rsbuild-shared';

export function SharedClientComponent() {
  return 'SharedClientComponent';
}

export async function sharedAction() {
  'use server';
  return sharedValue;
}
