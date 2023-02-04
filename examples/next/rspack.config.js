const path = require('path');
/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  builtins: {
    define: {
      __NEXT_DEFINE_ENV: 'true',
      'process.env.NODE_ENV': '"production"',
      'process.env.NEXT_RUNTIME': undefined,
      'process.env.__NEXT_MIDDLEWARE_MATCHERS': '[]',
      'process.env.__NEXT_MANUAL_CLIENT_BASE_PATH': 'false',
      'process.env.__NEXT_NEW_LINK_BEHAVIOR': 'true',
      'process.env.__NEXT_OPTIMISTIC_CLIENT_CACHE': 'true',
      'process.env.__NEXT_MIDDLEWARE_PREFETCH': '"flexible"',
      'process.env.__NEXT_CROSS_ORIGIN': undefined,
      'process.browser': 'true',
      'process.env.__NEXT_TEST_MODE': undefined,
      'process.env.__NEXT_TRAILING_SLASH': 'false',
      'process.env.__NEXT_BUILD_INDICATOR': 'true',
      'process.env.__NEXT_BUILD_INDICATOR_POSITION': '"bottom-right"',
      'process.env.__NEXT_STRICT_MODE': 'false',
      'process.env.__NEXT_STRICT_MODE_APP': 'false',
      'process.env.__NEXT_OPTIMIZE_FONTS': 'true',
      'process.env.__NEXT_OPTIMIZE_CSS': 'false',
      'process.env.__NEXT_SCRIPT_WORKERS': 'false',
      'process.env.__NEXT_SCROLL_RESTORATION': 'false',
      'process.env.__NEXT_IMAGE_OPTS': '{"deviceSizes":[640,750,828,1080,1200,1920,2048,3840],"imageSizes":[16,32,48,64,96,128,256,384],"path":"/_next/image","use":"default","dangerouslyAllowSVG":true,"unoptimized":false}',
      'process.env.__NEXT_ROUTER_BASEPATH': '""',
      'process.env.__NEXT_HAS_REWRITES': 'true',
      'process.env.__NEXT_I18N_SUPPORT': 'false',
      'process.env.__NEXT_I18N_DOMAINS': undefined,
      'process.env.__NEXT_ANALYTICS_ID': '""',
      'process.env.__NEXT_NO_MIDDLEWARE_URL_NORMALIZE': undefined,
      'process.env.__NEXT_MANUAL_TRAILING_SLASH': undefined,
      'process.env.__NEXT_HAS_WEB_VITALS_ATTRIBUTION': undefined,
      'process.env.__NEXT_WEB_VITALS_ATTRIBUTION': undefined
    }
  },
  entry: {
    'pages/_app': [
      'next/dist/pages/_app?page=/_app&loader=next-client-pages-loader',
      'next/dist/client/router'
    ],
    'pages/about': [
      './pages/about.js?pages=about&loader=next-client-pages-loader',
    ]
  },
  output: {
    publicPath: '/_next/',
    path: path.resolve(__dirname, '.next'),
    filename: 'static/chunks/[name]-[contenthash].js',
    chunkFilename: 'static/chunks/[name].js'
  },
  module: {
    rules: [
      {
        resourceQuery: /next-client-pages-loader/,
        use: [{
          loader: 'next/dist/build/webpack/loaders/next-client-pages-loader',
          options: {
            page: '_app', absolutePagePath: 'next/dist/pages/_app'
          }
        }],
        type: 'jsx'
      },
      {
        resourceQuery: /next-client-pages-loader/,
        use: [{
          loader: 'next/dist/build/webpack/loaders/next-client-pages-loader',
          options: {
            page: '_about', absolutePagePath: 'private-next-pages/about.js'
          }
        }],
        type: 'jsx'
      }
    ]
  }
}