'use client';

import { MixedClientBadge, mixedValue } from 'rsbuild-rsc-federation-shared';
import { composeExposeMessage, localPatternTag } from './local-patterns';

export default function ExposedButton() {
  return composeExposeMessage([
    'ExposedButton',
    typeof MixedClientBadge,
    mixedValue.kind,
    localPatternTag,
  ]);
}
