import url from 'url';

const __dirname = url.fileURLToPath(new URL('.', import.meta.url));
export default {
  context: __dirname,
  mode: 'development',
  entry: './src/entry.js',
};
