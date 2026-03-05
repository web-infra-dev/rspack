'use client';

import remoteButton from 'remote/Button';
import {
  SharedClientComponent,
  sharedValue,
} from 'rsbuild-rsc-federation-shared';

export default function ExposedButton() {
  return `${sharedValue}:${typeof remoteButton}:${typeof SharedClientComponent}`;
}
