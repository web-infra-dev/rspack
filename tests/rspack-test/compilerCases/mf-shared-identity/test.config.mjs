import { runNodeCase } from '@rspack/test-tools/helper/node-case';

export default [
  'unused-issuer-layer',
  'reachable-child-shared',
  'provider-layer',
  'consumer-layer',
  'fallback-scopes',
  'fallback-layers',
  'safe-scopes',
  'paired-scopes',
  'paired-layers',
  'paired-isolated-provider',
].map((scenario) => ({
  name: scenario,
  description: 'preserves shared identity: ' + scenario,
  async run() {
    await runNodeCase(new URL('./scenario.mjs', import.meta.url), [scenario]);
  },
}));
