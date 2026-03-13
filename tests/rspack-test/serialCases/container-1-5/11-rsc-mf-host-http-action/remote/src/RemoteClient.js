'use client';

import { remoteAction, remoteSecondaryAction } from './actions';
import { nestedAction } from './nestedActions';

export function RemoteClient() {
  return (
    <button
      type="button"
      onClick={() => {
        void remoteAction('from-client');
        void remoteSecondaryAction('from-client');
        void nestedAction('from-client');
      }}
    >
      Remote Client
    </button>
  );
}
