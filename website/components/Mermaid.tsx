import type { CSSProperties, PropsWithChildren } from 'react';
import './Mermaid.scss';

import mermaid, { type MermaidConfig } from 'mermaid';
import { useEffect, useId, useState } from 'react';

interface Props {
  style?: CSSProperties;
  title?: string;
  config?: MermaidConfig;
}
export default function Mermaid({
  style,
  children,
  title,
  config,
}: PropsWithChildren<Props>) {
  const id = useId();
  const [svg, setSvg] = useState('');
  const [renderError, setRenderError] = useState(false);

  async function renderMermaid2SVG() {
    // https://github.com/mermaid-js/mermaid/blob/1b40f552b20df4ab99a986dd58c9d254b3bfd7bc/packages/mermaid/src/docs/.vitepress/theme/Mermaid.vue#L53
    const hasDarkClass = document.documentElement.classList.contains('dark');

    const mermaidConfig: MermaidConfig = {
      securityLevel: 'loose',
      startOnLoad: false,
      theme: hasDarkClass ? 'dark' : 'default',
      ...config,
    };

    try {
      mermaid.initialize(mermaidConfig);

      const { svg } = await mermaid.render(
        id.replace(/:/g, ''),
        children as string,
      );

      setSvg(svg);
    } catch (_error) {
      setRenderError(true);
    }
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: safe
  useEffect(() => {
    renderMermaid2SVG();
  }, [children]);
  return (
    <>
      {renderError || !svg ? null : (
        <div style={style} className="rspack-mermaid rp-not-doc">
          <h3>{title}</h3>
          {/* biome-ignore lint/security/noDangerouslySetInnerHtml: safe */}
          <div dangerouslySetInnerHTML={{ __html: svg }} />
        </div>
      )}
    </>
  );
}
