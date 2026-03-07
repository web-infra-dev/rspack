import { RemoteClient } from './RemoteClient';
import { SharedFromPackage } from 'fake-shared-client';

export function RemoteApp({ label = 'remote' }) {
  return (
    <section>
      <h1>{label}</h1>
      <SharedFromPackage />
      <RemoteClient />
    </section>
  );
}
