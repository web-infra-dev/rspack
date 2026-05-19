import { createRequire } from 'node:module';
import { afterAll } from 'vitest';

if (process.env.RSPACK_PGO_PROFILE_DUMP) {
  const require = createRequire(import.meta.url);

  afterAll(() => {
    const binding = require(process.env.RSPACK_BINDING || '@rspack/binding');
    binding.writePgoProfile?.();
  });
}
