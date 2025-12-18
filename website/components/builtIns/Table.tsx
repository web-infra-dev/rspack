import Markdown from 'markdown-to-jsx';
import type { ReactNode } from 'react';
import { Table as BaseTable, Td, Th, Tr } from './mdx-components';

interface TableProps {
  children?: ReactNode[];
  body?: any[];
  header?: {
    name: string | JSX.Element;
    key: string;
    style?: React.CSSProperties;
    className?: string;
  }[];
  tableStyle?: Record<string, string>;
  className?: string;
}

// Use case example:
//
// import { Table } from '@builtIns';
//
// <Table
//   header={[
//     { name: 'Name', key: 'name' },
//     { name: 'Description', key: 'description' },
//   ]}
//   body={[
//     {
//       name: 'Foo',
//       description: 'Description1',
//     },
//     {
//       name: 'Bar',
//       description: 'Description2',
//     }
//   ]}
// />
export function Table(props: TableProps) {
  const { body = [], tableStyle, header = [] } = props;
  // Support markdown syntax in table cell
  const compiledValue = body.map((item: any) => {
    for (const key of Object.keys(item)) {
      if (typeof item[key] === 'string') {
        item[key] = <Markdown>{item[key]}</Markdown>;
      }
    }
    return item;
  });

  const renderHeaderItem = (name: string | JSX.Element) => {
    if (typeof name === 'string') {
      return <Markdown>{name}</Markdown>;
    }
    return name;
  };

  // generate table tag
  return (
    <BaseTable style={tableStyle} className={props.className}>
      <thead>
        <Tr>
          {header.map(item => (
            <Th key={item.key} style={item.style}>
              {renderHeaderItem(item.name)}
            </Th>
          ))}
        </Tr>
      </thead>
      <tbody>
        {compiledValue.map((item: any, index: number) => {
          const key = `row-${index}`;
          return (
            <Tr key={key}>
              {header.map(headerItem => (
                <Td key={headerItem.key}>{item[headerItem.key]}</Td>
              ))}
            </Tr>
          );
        })}
      </tbody>
    </BaseTable>
  );
}
