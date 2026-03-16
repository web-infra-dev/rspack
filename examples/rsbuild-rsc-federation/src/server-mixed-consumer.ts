'use server';

import remoteButton from 'remote/button';
import { consumeMixedPatterns as consumeRemoteMixedModule } from 'remote/server-mixed';
import { mixedValue } from 'rsbuild-rsc-federation-shared';
import { mixedServerAction } from 'rsbuild-rsc-federation-shared/server-actions';
import { composeExposeMessage, describeModuleType } from './local-patterns';

export async function consumeMixedPatterns() {
  const remoteMixed = await consumeRemoteMixedModule();
  const mixedResponse = await mixedServerAction('server-mixed');
  return composeExposeMessage([
    remoteMixed,
    mixedResponse,
    mixedValue.kind,
    describeModuleType(remoteButton),
  ]);
}
