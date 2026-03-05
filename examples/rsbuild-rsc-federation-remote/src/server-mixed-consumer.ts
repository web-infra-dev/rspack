'use server';

import remoteButton from 'remote/Button';
import { mixedValue } from 'rsbuild-rsc-federation-shared';
import { mixedServerAction } from 'rsbuild-rsc-federation-shared/server-actions';
import { composeExposeMessage, describeModuleType } from './local-patterns';

export async function consumeMixedPatterns() {
  const mixedResponse = await mixedServerAction('server-mixed');
  return composeExposeMessage([
    mixedResponse,
    mixedValue.kind,
    describeModuleType(remoteButton),
  ]);
}
