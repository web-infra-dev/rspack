import { Children } from 'react';
import type { PropsWithChildren } from 'react';

interface Props {
  titles: string[];
}

export default function Columns({
  children,
  titles = [],
}: PropsWithChildren<Props>) {
  return (
    <div className="flex flex-wrap gap-4">
      {Children.map(children, (child, index) => {
        return <Column title={titles[index]}>{child}</Column>;
      })}
    </div>
  );
}

interface ColumnProps {
  title?: string;
}

export function Column({ title, children }: PropsWithChildren<ColumnProps>) {
  return (
    <div className="w-80 flex-auto m-auto" style={{ marginTop: 0 }}>
      {title && <div className="font-bold text-center">{title}</div>}
      {children}
    </div>
  );
}
