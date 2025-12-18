import type { ComponentProps } from 'react';

export const Table = (props: ComponentProps<'table'>) => {
  return (
    <div className={`${props.className || ''}`}>
      <table {...props} />
    </div>
  );
};

export const Tr = (props: ComponentProps<'tr'>) => {
  return <tr {...props} className={`${props.className || ''}`} />;
};

export const Td = (props: ComponentProps<'td'>) => {
  return <td {...props} className={`${props.className || ''}`} />;
};

export const Th = (props: ComponentProps<'th'>) => {
  return <th {...props} className={`${props.className || ''}`} />;
};
