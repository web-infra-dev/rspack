import { RemoteClient } from './RemoteClient';

export function RemoteApp({ label = 'remote' }) {
  return (
    <section>
      <h1>{label}</h1>
      <RemoteClient />
    </section>
  );
}
