module.exports = function (templateParams) {
  const version = parseInt(process.version.match(/^v(\d+)/)[1]);

  if (typeof URL !== 'function') {
    throw new Error('Error');
  }

  if (typeof URLSearchParams !== 'function') {
    throw new Error('Error');
  }

  if (version >= 11 && typeof TextEncoder !== 'function') {
    throw new Error('Error');
  }

  if (version >= 11 && typeof TextDecoder !== 'function') {
    throw new Error('Error');
  }

  return 'templateParams keys: "' + Object.keys(templateParams).join(',') + '"';
};
