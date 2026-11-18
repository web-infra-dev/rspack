const { container } = require('@rspack/core');

module.exports = {
  target: 'async-node',
  plugins: [
    new container.ModuleFederationPlugin({ name: 'low-level-script-host' }),
    new container.ContainerReferencePlugin({
      enhanced: true,
      remoteType: 'script',
      remotes: {
        standalone: 'lowLevelScriptRemote@https://unused.invalid/remote.js',
      },
    }),
  ],
};
