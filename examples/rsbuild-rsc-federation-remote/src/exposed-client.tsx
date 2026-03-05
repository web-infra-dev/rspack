'use client';

import remoteButton from 'remote/Button';
import { MixedClientBadge, mixedValue } from 'rsbuild-rsc-federation-shared';
import {
  composeExposeMessage,
  describeModuleType,
  localPatternTag,
} from './local-patterns';

export default function ExposedButton() {
  return composeExposeMessage([
    'ExposedButton',
    describeModuleType(remoteButton),
    describeModuleType(MixedClientBadge),
    mixedValue.kind,
    localPatternTag,
  ]);
}
