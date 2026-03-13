'use server';

export async function remoteAction(input) {
  return `remote-action:${input}`;
}

export async function remoteSecondaryAction(input) {
  return `remote-secondary:${input}`;
}
