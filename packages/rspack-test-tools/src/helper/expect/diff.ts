const CURRENT_CWD = process.cwd();

const quoteMeta = (str: string) => str.replace(/[-[\]\\/{}()*+?.^$|]/g, '\\$&');
const cwdRegExp = new RegExp(
  `${quoteMeta(CURRENT_CWD)}((?:\\\\)?(?:[a-zA-Z.\\-_]+\\\\)*)`,
  'g',
);
const escapedCwd = JSON.stringify(CURRENT_CWD).slice(1, -1);
const escapedCwdRegExp = new RegExp(
  `${quoteMeta(escapedCwd)}((?:\\\\\\\\)?(?:[a-zA-Z.\\-_]+\\\\\\\\)*)`,
  'g',
);

export const normalizeDiff = (diff: { value: string }) => {
  let normalizedStr: string = diff.value;
  if (CURRENT_CWD.startsWith('/')) {
    normalizedStr = normalizedStr.replace(
      new RegExp(quoteMeta(CURRENT_CWD), 'g'),
      '<cwd>',
    );
  } else {
    normalizedStr = normalizedStr.replace(
      cwdRegExp,
      (_, g) => `<cwd>${g.replace(/\\/g, '/')}`,
    );
    normalizedStr = normalizedStr.replace(
      escapedCwdRegExp,
      (_, g) => `<cwd>${g.replace(/\\\\/g, '/')}`,
    );
  }
  normalizedStr = normalizedStr.replace(
    /@@ -\d+,\d+ \+\d+,\d+ @@/g,
    '@@ ... @@',
  );
  return normalizedStr;
};
