'use server';

export async function remoteAction(input) {
  return `remote-action:${input}`;
}
