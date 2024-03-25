import { ReactNode } from 'react';
import Markdown from 'markdown-to-jsx';
import {
  Table as ModernTable,
  Th as ModernTableHead,
  Tr as ModernTableRow,
  Td as ModernTableData,
} from './mdx-components';

interface TableProps {
  children?: ReactNode[];
  body?: any[];
  header?: { name: string | JSX.Element; key: string }[];
  tableStyle?: Record<string, string>;
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
//       name: 'Modern.js',
//       description: 'A JavaScript framework for the modern web.',
//     },
//     {
//       name: 'Modern.js Doc Tools',
//       description: 'A tool for building documentation sites.',
//     }
//   ]}
// />
export function Table(props: TableProps) {
  const { body = [], tableStyle, header = [] } = props;
  // Support markdown syntax in table cell
  const compiledValue = body.map((item: any) => {
    Object.keys(item).forEach((key) => {
      if (typeof item[key] === 'string') {
        item[key] = <Markdown>{item[key]}</Markdown>;
      }
    });
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
    <ModernTable style={tableStyle}>
      <ModernTableRow>
        {header.map((item) => (
          <ModernTableHead key={item.key}>
            {renderHeaderItem(item.name)}
          </ModernTableHead>
        ))}
      </ModernTableRow>
      <tbody>
        {compiledValue.map((item: any, index: number) => (
          <ModernTableRow key={index}>
            {header.map((headerItem) => (
              <ModernTableData key={headerItem.key}>
                {item[headerItem.key]}
              </ModernTableData>
            ))}
          </ModernTableRow>
        ))}
      </tbody>
    </ModernTable>
  );
}
