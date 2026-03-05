'use client';

import remoteButton from 'remote/Button';
import {
  MixedClientBadge,
  mixedValue,
  sharedValue,
} from 'rsbuild-rsc-federation-shared';
import {
  composeExposeMessage,
  describeModuleType,
  localPatternTag,
} from './local-patterns';

export function describeComposedExpose() {
  return composeExposeMessage([
    'ComposedExpose',
    sharedValue,
    mixedValue.scope,
    describeModuleType(MixedClientBadge),
    describeModuleType(remoteButton),
    localPatternTag,
  ]);
}

export default function ComposedExpose() {
  return describeComposedExpose();
}
