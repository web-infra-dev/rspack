import cases from './cases.mjs';

export default function () {
  return {
    name: 'provide-test-shared-modules',
    beforeInit(args) {
      args.userOptions.shared = Object.fromEntries(
        cases.map(({ name, version }) => [
          name,
          {
            version,
            scope: 'default',
            lib: () => name,
            shareConfig: { requiredVersion: false },
          },
        ]),
      );
      return args;
    },
  };
}
