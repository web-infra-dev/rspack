module.exports = async () => {
  const { default: remote } = await import('peer/value');
  const { default: local } = await import('singleton-lib');
  return { remote: remote.provider, host: local.provider, sameObject: remote === local };
};
