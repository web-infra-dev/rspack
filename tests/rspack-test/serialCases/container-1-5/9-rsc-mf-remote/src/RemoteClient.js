'use client';

import { remoteAction } from './actions';

export function RemoteClient() {
  return (
    <button type="button" onClick={() => remoteAction('from-client')}>
      Remote Client
    </button>
  );
}
