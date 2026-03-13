import { RemoteNestedMixed } from './RemoteNestedMixed';

export function RemoteApp({ label = 'remote' }) {
  const manifest = __rspack_rsc_manifest__;
  const actionCount = Object.keys(manifest.serverManifest || {}).length;

  return (
    <section data-action-count={actionCount}>
      <h1>{label}</h1>
      <RemoteNestedMixed label={`${label}-nested`} />
    </section>
  );
}
