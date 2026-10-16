const contents = {
  '?empty': '',
  '?newlines': '\n\n',
  '?crlf': '\r\n',
  '?whitespace': ' \t',
  '?unicode': '// 你好 🌍\n',
  '?mapped': 'export const value = "mapped";',
};

export default function loader() {
  const content = contents[this.resourceQuery];
  this.callback(
    null,
    content,
    this.resourceQuery === '?mapped'
      ? {
          version: 3,
          sources: ['original.js'],
          sourcesContent: [content],
          names: [],
          mappings: '',
        }
      : undefined,
  );
}
