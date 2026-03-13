import { RemoteClient } from './RemoteClient';
import { getServerOnlyInfo } from './ServerOnlyInfo';

export function RemoteServerCard({ label = 'server-card' }) {
  return (
    <article data-server-only={getServerOnlyInfo()}>
      <h2>{label}</h2>
      <RemoteClient />
    </article>
  );
}
