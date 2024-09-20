import { Children } from 'react';
import type { CSSProperties, PropsWithChildren } from 'react';

interface Props {
  titles?: string[];
  styles?: CSSProperties[];
}

export default function Columns({
  children,
  titles = [],
  styles = [],
}: PropsWithChildren<Props>) {
  return (
    <div className="flex flex-wrap gap-4">
      {Children.map(children, (child, index) => {
        const title = titles[index];
        const style = styles[index];
        return (
          <Column key={title} title={title} style={style}>
            {child}
          </Column>
        );
      })}
    </div>
  );
}

interface ColumnProps {
  title?: string;
  style?: CSSProperties;
}

export function Column({
  title,
  children,
  style,
}: PropsWithChildren<ColumnProps>) {
  return (
    <div className="flex-auto m-auto" style={{ marginTop: 0, ...style }}>
      {title && <div className="font-bold text-center">{title}</div>}
      {children}
    </div>
  );
}
