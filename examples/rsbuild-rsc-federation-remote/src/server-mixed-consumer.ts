'use server';

import { mixedValue } from 'rsbuild-rsc-federation-shared';
import { mixedServerAction } from 'rsbuild-rsc-federation-shared/server-actions';
import { composeExposeMessage } from './local-patterns';

export async function consumeMixedPatterns() {
  const mixedResponse = await mixedServerAction('server-mixed');
  return composeExposeMessage([mixedResponse, mixedValue.kind]);
}
