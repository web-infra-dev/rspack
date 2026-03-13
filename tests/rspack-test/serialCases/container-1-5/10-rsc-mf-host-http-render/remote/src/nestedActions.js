'use server';

export async function nestedAction(input) {
  return `nested-action:${input}`;
}
