import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dir = path.dirname(fileURLToPath(import.meta.url));
const load = (impl) => {
  const file = path.join(dir, `results-${impl}.json`);
  return fs.existsSync(file)
    ? JSON.parse(fs.readFileSync(file, 'utf-8'))
    : null;
};
const oldRun = load('old');
const newRun = load('new');
if (!oldRun || !newRun) {
  console.error(
    'missing results-old.json / results-new.json — run bench.mjs first',
  );
  process.exit(1);
}

const labels = newRun.results.map((r) => r.label);
const series = (run, pick) => run.results.map(pick);

function lineChart({ title, unit, yValues, height = 260, width = 640 }) {
  const pad = { top: 20, right: 130, bottom: 34, left: 46 };
  const iw = width - pad.left - pad.right;
  const ih = height - pad.top - pad.bottom;
  const yMax = Math.max(...yValues.flatMap((s) => s.values)) * 1.15 || 1;
  const x = (i) => pad.left + (i / (labels.length - 1)) * iw;
  const y = (v) => pad.top + ih - (v / yMax) * ih;
  const ticks = 4;
  const grid = Array.from({ length: ticks + 1 }, (_, t) => {
    const v = (yMax / ticks) * t;
    return `<line x1="${pad.left}" x2="${pad.left + iw}" y1="${y(v)}" y2="${y(v)}" class="grid"/>
      <text x="${pad.left - 8}" y="${y(v) + 4}" class="axis" text-anchor="end">${Math.round(v)}</text>`;
  }).join('');
  const xAxis = labels
    .map(
      (l, i) =>
        `<text x="${x(i)}" y="${height - 10}" class="axis" text-anchor="middle">${l}</text>`,
    )
    .join('');
  const body = yValues
    .map(({ name, values, cls }) => {
      const points = values.map((v, i) => `${x(i)},${y(v)}`).join(' ');
      const markers = values
        .map(
          (
            v,
            i,
          ) => `<circle cx="${x(i)}" cy="${y(v)}" r="4" class="marker ${cls}"/>
          <circle cx="${x(i)}" cy="${y(v)}" r="12" class="hit" data-tip="${name} · ${labels[i]}: ${v}${unit}"/>`,
        )
        .join('');
      const endLabel = `<text x="${x(values.length - 1) + 10}" y="${y(values[values.length - 1]) + 4}" class="series-label ${cls}">${name}</text>`;
      return `<polyline points="${points}" class="line ${cls}"/>${markers}${endLabel}`;
    })
    .join('');
  return `<figure>
    <figcaption>${title}</figcaption>
    <svg viewBox="0 0 ${width} ${height}" role="img" aria-label="${title}">
      ${grid}${xAxis}${body}
    </svg>
  </figure>`;
}

const p50 = (r, scenario) => Math.round(r[scenario].latency.p50);
const p95 = (r, scenario) => Math.round(r[scenario].latency.p95);
const req = (r, scenario) => r[scenario].cssRequests.p50;

const tableRows = labels
  .map((label, i) => {
    const o = oldRun.results[i];
    const n = newRun.results[i];
    return `<tr>
      <th scope="row">${label}<span class="dim"> · ${(o.byteSize / 1024).toFixed(0)} KB</span></th>
      <td>${p50(o, 'jsOnly')} / ${p95(o, 'jsOnly')}</td><td>${req(o, 'jsOnly')}</td>
      <td>${p50(n, 'jsOnly')} / ${p95(n, 'jsOnly')}</td><td>${req(n, 'jsOnly')}</td>
      <td>${p50(o, 'cssEdit')} / ${p95(o, 'cssEdit')}</td><td>${req(o, 'cssEdit')}</td>
      <td>${p50(n, 'cssEdit')} / ${p95(n, 'cssEdit')}</td><td>${req(n, 'cssEdit')}</td>
    </tr>`;
  })
  .join('');

const oldJsOnly = series(oldRun, (r) => p50(r, 'jsOnly'));
const worstOld = Math.max(...oldJsOnly);
const worstOldLabel = labels[oldJsOnly.indexOf(worstOld)];
const worstNew = Math.max(...series(newRun, (r) => p50(r, 'jsOnly')));

const html = `<!doctype html>
<html lang="zh">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>CSS HMR perf — precise updates vs @rspack/core@${oldRun.version}</title>
<style>
.viz-root {
  color-scheme: light;
  --surface-1: #fcfcfb; --text-primary: #0b0b0b; --text-secondary: #52514e;
  --grid: #e4e3df; --series-old: #2a78d6; --series-new: #1baf7a;
  font: 14px/1.5 -apple-system, "Segoe UI", sans-serif;
  background: var(--surface-1); color: var(--text-primary);
  max-width: 820px; margin: 0 auto; padding: 24px 16px 48px;
}
@media (prefers-color-scheme: dark) {
  :root:where(:not([data-theme="light"])) .viz-root {
    color-scheme: dark;
    --surface-1: #1a1a19; --text-primary: #ffffff; --text-secondary: #c3c2b7;
    --grid: #33322f; --series-old: #3987e5; --series-new: #199e70;
  }
}
:root[data-theme="dark"] .viz-root {
  color-scheme: dark;
  --surface-1: #1a1a19; --text-primary: #ffffff; --text-secondary: #c3c2b7;
  --grid: #33322f; --series-old: #3987e5; --series-new: #199e70;
}
html, body { margin: 0; background: var(--surface-1); }
h1 { font-size: 20px; margin: 0 0 4px; }
.meta { color: var(--text-secondary); margin: 0 0 24px; }
.tiles { display: flex; gap: 12px; flex-wrap: wrap; margin-bottom: 28px; }
.tile { border: 1px solid var(--grid); border-radius: 8px; padding: 12px 16px; flex: 1 1 220px; }
.tile .num { font-size: 26px; font-weight: 650; }
.tile .cap { color: var(--text-secondary); }
figure { margin: 0 0 28px; }
figcaption { font-weight: 600; margin-bottom: 8px; }
svg { width: 100%; height: auto; display: block; }
.grid { stroke: var(--grid); stroke-width: 1; }
.axis { fill: var(--text-secondary); font-size: 11px; }
.line { fill: none; stroke-width: 2; }
.line.old, .marker.old { stroke: var(--series-old); }
.marker.old { fill: var(--series-old); }
.line.new, .marker.new { stroke: var(--series-new); }
.marker.new { fill: var(--series-new); }
.series-label { font-size: 12px; font-weight: 600; }
.series-label.old { fill: var(--series-old); }
.series-label.new { fill: var(--series-new); }
.hit { fill: transparent; cursor: default; }
.legend { display: flex; gap: 16px; color: var(--text-secondary); margin-bottom: 8px; }
.legend .swatch { display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 6px; }
table { border-collapse: collapse; width: 100%; overflow-x: auto; display: block; }
th, td { border-bottom: 1px solid var(--grid); padding: 6px 10px; text-align: right; white-space: nowrap; }
th[scope=row] { text-align: left; }
thead th { color: var(--text-secondary); font-weight: 600; }
.dim { color: var(--text-secondary); font-weight: 400; }
.note { color: var(--text-secondary); font-size: 13px; margin-top: 20px; }
#tooltip { position: fixed; pointer-events: none; background: var(--text-primary); color: var(--surface-1);
  padding: 4px 8px; border-radius: 4px; font-size: 12px; display: none; z-index: 10; }
</style>
</head>
<body class="viz-root">
<h1>CSS HMR：精确更新 vs 全量重载</h1>
<p class="meta">old = @rspack/core@${oldRun.version}（npm，每次更新重载已更新 chunk 的样式表） · new = 本分支 workspace 构建（manifest 驱动的精确 CSS 更新） · 每场景 ${newRun.rounds} 轮取 p50，延迟自浏览器收到首个 hot-update 请求起算（不含编译耗时）</p>

<div class="tiles">
  <div class="tile"><div class="num">${worstOld}ms → ${worstNew}ms</div><div class="cap">js-only 更新的最差 p50 浏览器侧耗时（${worstOldLabel} 样式表）</div></div>
  <div class="tile"><div class="num">1 → 0</div><div class="cap">js-only 更新触发的 CSS 请求数（每次更新）</div></div>
  <div class="tile"><div class="num">2 → 1</div><div class="cap">CSS 修改触发的 CSS 请求数（旧实现双重拉取）</div></div>
</div>

<div class="legend">
  <span><span class="swatch" style="background: var(--series-old)"></span>old ${oldRun.version}</span>
  <span><span class="swatch" style="background: var(--series-new)"></span>new（本分支）</span>
</div>

${lineChart({
  title: 'js-only 更新：浏览器侧应用耗时 p50（ms，越低越好）',
  unit: 'ms',
  yValues: [
    {
      name: 'old',
      cls: 'old',
      values: series(oldRun, (r) => p50(r, 'jsOnly')),
    },
    {
      name: 'new',
      cls: 'new',
      values: series(newRun, (r) => p50(r, 'jsOnly')),
    },
  ],
})}

${lineChart({
  title: 'CSS 修改：浏览器侧应用耗时 p50（ms，越低越好）',
  unit: 'ms',
  yValues: [
    {
      name: 'old',
      cls: 'old',
      values: series(oldRun, (r) => p50(r, 'cssEdit')),
    },
    {
      name: 'new',
      cls: 'new',
      values: series(newRun, (r) => p50(r, 'cssEdit')),
    },
  ],
})}

<figure>
<figcaption>完整数据（延迟为 p50 / p95 ms；请求数为每轮 CSS 请求 p50）</figcaption>
<table>
  <thead>
    <tr><th></th><th colspan="4">js-only 更新</th><th colspan="4">CSS 修改</th></tr>
    <tr><th>规模</th><th>old 延迟</th><th>old 请求</th><th>new 延迟</th><th>new 请求</th><th>old 延迟</th><th>old 请求</th><th>new 延迟</th><th>new 请求</th></tr>
  </thead>
  <tbody>${tableRows}</tbody>
</table>
</figure>

<p class="note">方法：extract CSS 单 chunk，规模为生成样式表的目标字节数（0.5MB–5MB，步进 0.5MB）。js-only 轮只改 marker.js；CSS 轮只改一条规则的颜色。旧实现的 js-only 耗时随样式表体积线性增长（重载 + 重解析整张样式表），新实现与体积无关且零请求。本地为 dev-profile binding，故耗时不含编译阶段。</p>

<div id="tooltip"></div>
<script>
const tip = document.getElementById('tooltip');
document.querySelectorAll('.hit').forEach((el) => {
  el.addEventListener('mouseenter', (e) => {
    tip.textContent = el.dataset.tip;
    tip.style.display = 'block';
  });
  el.addEventListener('mousemove', (e) => {
    tip.style.left = e.clientX + 12 + 'px';
    tip.style.top = e.clientY + 12 + 'px';
  });
  el.addEventListener('mouseleave', () => { tip.style.display = 'none'; });
});
</script>
</body>
</html>
`;

fs.writeFileSync(path.join(dir, 'report.html'), html);
console.log('written report.html');
