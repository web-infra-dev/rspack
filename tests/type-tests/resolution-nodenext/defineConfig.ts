import { defineConfig } from '@rspack/cli';

defineConfig({
  mode: 'development',
});

defineConfig([
  {
    mode: 'development',
  },
  {
    mode: 'production',
  },
]);

defineConfig(() => ({
  mode: 'development',
}));

defineConfig((env, argv) => ({
  mode:
    env.production || argv.mode === 'production' ? 'production' : 'development',
}));

defineConfig(() => [
  {
    mode: 'development',
  },
  {
    mode: 'production',
  },
]);

defineConfig(async () => ({
  mode: 'development',
}));

defineConfig(async () => [
  {
    mode: 'development',
  },
  {
    mode: 'production',
  },
]);

// @ts-expect-error invalid mode
defineConfig({
  mode: 'invalid',
});

// @ts-expect-error invalid mode
defineConfig(() => ({
  mode: 'invalid',
}));

// @ts-expect-error invalid mode
defineConfig(async () => ({
  mode: 'invalid',
}));
