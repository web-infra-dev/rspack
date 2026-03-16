import { renderToString } from 'react-dom/server';

export function renderSsrShell() {
  return renderToString(<div data-ssr-shell="ready">ssr shell</div>);
}
