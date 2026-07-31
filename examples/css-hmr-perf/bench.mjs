import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';
import { generateCss, setMarker, setProbeColor } from './gen-css.mjs';

const dir = path.dirname(fileURLToPath(import.meta.url));
const impl = process.argv.includes('--old') ? 'old' : 'new';
const corePkg = impl === 'old' ? 'rspack-npm' : '@rspack/core';
const { rspack } = await import(corePkg);
const { RspackDevServer } = await import('@rspack/dev-server');

const SIZES = Array.from({ length: 10 }, (_, i) => {
  const mb = (i + 1) * 0.5;
  return { label: `${mb}MB`, bytes: mb * 1_000_000 };
});
const WARMUP = 2;
const ROUNDS = 12;
// the legacy runtime reloads stylesheets after the js update settles (with a
// debounce), keep the window open long enough to observe it
const ROUND_TAIL_MS = 700;

function makeConfig(port) {
  return {
    context: dir,
    mode: 'development',
    entry: { main: './src/index.js' },
    stats: 'errors-warnings',
    plugins: [
      new rspack.HtmlRspackPlugin({
        template: './src/index.html',
        inject: 'body',
      }),
      new rspack.CssExtractRspackPlugin(),
    ],
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'javascript/auto',
          use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
        },
      ],
    },
    devServer: {
      port,
      hot: true,
      open: false,
      client: { logging: 'error', overlay: false },
    },
  };
}

const stats = (values) => {
  const sorted = [...values].sort((a, b) => a - b);
  const pick = (q) =>
    sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))];
  return {
    mean: values.reduce((a, b) => a + b, 0) / values.length,
    p50: pick(0.5),
    p95: pick(0.95),
  };
};

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function benchSize(browser, size, port) {
  const shape = generateCss(size);
  setMarker('step-0');
  console.log(
    `[${impl}] ${size.label}: ${shape.ruleCount} rules, ${(shape.byteSize / 1024).toFixed(0)} KB`,
  );

  const compiler = rspack(makeConfig(port));
  const server = new RspackDevServer(makeConfig(port).devServer, compiler);
  await server.start();

  const page = await browser.newPage();
  // per-round trace of hot-update / css requests, timestamped on arrival
  let trace = [];
  page.on('request', (request) => {
    const url = request.url();
    if (url.includes('hot-update') || /\.css(\?|$)/.test(url)) {
      trace.push({ url, at: Date.now() });
    }
  });
  await page.goto(`http://localhost:${port}`);
  await page.waitForSelector('#root:has-text("step-0")');

  const round = async (mutate, settle) => {
    trace = [];
    mutate();
    await settle();
    const applied = Date.now();
    await sleep(ROUND_TAIL_MS);
    const firstHotUpdate = trace.find((r) => r.url.includes('hot-update'));
    // measured from the first hot-update request so server-side rebuild time
    // (dev-profile vs release binding) stays out of the comparison
    const latency = firstHotUpdate ? applied - firstHotUpdate.at : NaN;
    const cssRequests = trace.filter(
      (r) => /\.css(\?|$)/.test(r.url) && !r.url.includes('hot-update'),
    ).length;
    return { latency, cssRequests };
  };

  const jsOnly = [];
  const cssEdit = [];
  for (let i = 0; i < WARMUP + ROUNDS; i += 1) {
    const marker = `step-${i + 1}`;
    const js = await round(
      () => setMarker(marker),
      () =>
        page.waitForFunction(
          (expected) =>
            document.getElementById('root').textContent === expected,
          marker,
          { polling: 'raf' },
        ),
    );
    const color = i % 2 === 0 ? 'rgb(0, 128, 0)' : 'rgb(255, 0, 0)';
    const css = await round(
      () => setProbeColor(color),
      () =>
        page.waitForFunction(
          (expected) =>
            getComputedStyle(document.getElementById('probe')).color ===
            expected,
          color,
          { polling: 'raf' },
        ),
    );
    if (i >= WARMUP) {
      jsOnly.push(js);
      cssEdit.push(css);
    }
  }

  await page.close();
  await server.stop();

  return {
    label: size.label,
    ruleCount: shape.ruleCount,
    byteSize: shape.byteSize,
    jsOnly: {
      latency: stats(jsOnly.map((r) => r.latency)),
      cssRequests: stats(jsOnly.map((r) => r.cssRequests)),
    },
    cssEdit: {
      latency: stats(cssEdit.map((r) => r.latency)),
      cssRequests: stats(cssEdit.map((r) => r.cssRequests)),
    },
  };
}

const browser = await chromium.launch();
const results = [];
for (const [i, size] of SIZES.entries()) {
  results.push(await benchSize(browser, size, 8380 + i));
}
await browser.close();

const version = JSON.parse(
  fs.readFileSync(
    path.join(dir, 'node_modules', corePkg, 'package.json'),
    'utf-8',
  ),
).version;
const out = { impl, corePkg, version, rounds: ROUNDS, results };
fs.writeFileSync(
  path.join(dir, `results-${impl}.json`),
  JSON.stringify(out, null, 2),
);
console.log(`\nwritten results-${impl}.json`);
for (const r of results) {
  console.log(
    `${r.label.padEnd(10)} js-only: ${r.jsOnly.latency.p50.toFixed(0)}ms / ${r.jsOnly.cssRequests.p50} css req` +
      `   css-edit: ${r.cssEdit.latency.p50.toFixed(0)}ms / ${r.cssEdit.cssRequests.p50} css req`,
  );
}
