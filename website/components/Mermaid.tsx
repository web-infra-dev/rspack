import type { CSSProperties, PropsWithChildren } from 'react';
import './Mermaid.scss';

interface Props {
  style?: CSSProperties;
  title?: string;
}
export default function Mermaid({
  style,
  children,
  title,
}: PropsWithChildren<Props>) {
  return (
    <div style={style} className="rspack-mermaid">
      <h3>{title}</h3>
      {children}
    </div>
  );
}
