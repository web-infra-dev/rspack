'use server';

import remoteButton from 'remote/button';
import { mixedValue, sharedValue } from 'rsbuild-rsc-federation-shared';
import {
  mixedServerAction,
  sharedAction,
} from 'rsbuild-rsc-federation-shared/server-actions';
import { composeExposeMessage, describeModuleType } from './local-patterns';

export async function consumeRemoteAndShared() {
  await sharedAction();
  const mixedResponse = await mixedServerAction('consumer');
  return composeExposeMessage([
    sharedValue,
    mixedResponse,
    mixedValue.kind,
    describeModuleType(remoteButton),
  ]);
}
