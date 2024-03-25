import { ComponentProps } from 'react';

export const Table = (props: ComponentProps<'table'>) => {
  return (
    <div className="w-full overflow-x-auto">
      <table
        {...props}
        className="table w-full border-collapse text-base my-5 leading-7 border-gray-light-2"
      />
    </div>
  );
};

export const Tr = (props: ComponentProps<'tr'>) => {
  return (
    <tr
      {...props}
      className="border border-solid transition-colors duration-500 even:bg-soft border-gray-light-2"
    />
  );
};

export const Td = (props: ComponentProps<'td'>) => {
  return (
    <td
      {...props}
      className="border border-solid  px-4 py-2 border-gray-light-2"
    />
  );
};

export const Th = (props: ComponentProps<'th'>) => {
  return (
    <th
      {...props}
      className="border border-solid px-4 py-2 text-text-1 text-base font-semibold border-gray-light-2"
    />
  );
};
