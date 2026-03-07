import { RemoteClient } from './RemoteClient';
import { RemoteServerCard } from './RemoteServerCard';

export function RemoteNestedMixed({ label = 'nested-mixed' }) {
  return (
    <section>
      <RemoteServerCard label={label} />
      <RemoteClient />
    </section>
  );
}
