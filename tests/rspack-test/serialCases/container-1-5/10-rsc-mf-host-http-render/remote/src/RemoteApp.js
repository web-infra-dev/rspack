import { RemoteNestedMixed } from './RemoteNestedMixed';

export function RemoteApp({ label = 'remote' }) {
  const manifest = __rspack_rsc_manifest__;
  const clientCount = Object.keys(manifest.clientManifest || {}).length;

  return (
    <section data-client-count={clientCount}>
      <h1>{label}</h1>
      <RemoteNestedMixed label={`${label}-nested`} />
    </section>
  );
}
