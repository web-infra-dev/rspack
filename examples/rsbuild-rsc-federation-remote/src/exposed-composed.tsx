'use client';

import {
  MixedClientBadge,
  mixedValue,
  sharedValue,
} from 'rsbuild-rsc-federation-shared';
import { composeExposeMessage, localPatternTag } from './local-patterns';

export function describeComposedExpose() {
  return composeExposeMessage([
    'ComposedExpose',
    sharedValue,
    mixedValue.scope,
    typeof MixedClientBadge,
    localPatternTag,
  ]);
}

export default function ComposedExpose() {
  return describeComposedExpose();
}
