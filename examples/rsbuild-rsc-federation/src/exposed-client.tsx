'use client';

import remoteButton from 'remote/Button';

export default function ExposedButton() {
  return `ExposedButton:${typeof remoteButton}`;
}
