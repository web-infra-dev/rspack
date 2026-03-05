'use server';

import remoteButton from 'remote/Button';
import { sharedAction, sharedValue } from 'rsbuild-rsc-federation-shared';

export async function consumeRemoteAndShared() {
  await sharedAction();
  return `${sharedValue}:${typeof remoteButton}`;
}
