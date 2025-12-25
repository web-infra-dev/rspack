export const normalizeStats = (stats: { value: string }): string => {
  return (
    stats.value
      // CHANGE: Remove potential line break and "|" caused by long text
      .replace(/((ERROR|WARNING)([\s\S](?!╭|├))*?)(\n {2}│ )/g, '$1')
      // CHANGE: Update the regular expression to replace the 'Rspack' version string
      .replace(/Rspack [^ )]+(\)?) compiled/g, 'Rspack x.x.x$1 compiled')
      .replace(/(\w)\\(\w)/g, '$1/$2')
      .replace(/, additional resolving: X ms/g, '')
      .replace(/Unexpected identifier '.+?'/g, 'Unexpected identifier')
  );
};
