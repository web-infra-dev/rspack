'use server';

import remoteButton from 'remote/button';
import { consumeRemoteAndShared as consumeRemoteServerModule } from 'remote/consumer';
import { mixedValue, sharedValue } from 'rsbuild-rsc-federation-shared';
import {
  mixedServerAction,
  sharedAction,
} from 'rsbuild-rsc-federation-shared/server-actions';
import { composeExposeMessage, describeModuleType } from './local-patterns';

export async function consumeRemoteAndShared() {
  const remoteConsumed = await consumeRemoteServerModule();
  await sharedAction();
  const mixedResponse = await mixedServerAction('consumer');
  return composeExposeMessage([
    sharedValue,
    remoteConsumed,
    mixedResponse,
    mixedValue.kind,
    describeModuleType(remoteButton),
  ]);
}
