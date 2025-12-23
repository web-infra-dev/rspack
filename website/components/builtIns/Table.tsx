import { getCustomMDXComponent, Link } from '@rspress/core/theme';
import Markdown, { type MarkdownToJSX } from 'markdown-to-jsx';
import type { JSX, ReactNode } from 'react';

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

const MarkdownOptions = {
  overrides: {
    a: {
      component: Link,
    },
  },
} satisfies MarkdownToJSX.Options;

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
        item[key] = <Markdown options={MarkdownOptions}>{item[key]}</Markdown>;
      }
    }
    return item;
  });

  const { table: Table } = getCustomMDXComponent();

  const renderHeaderItem = (name: string | JSX.Element) => {
    if (typeof name === 'string') {
      return <Markdown options={MarkdownOptions}>{name}</Markdown>;
    }
    return name;
  };

  // generate table tag
  return (
    <Table style={tableStyle} className={props.className}>
      <thead>
        <tr>
          {header.map(item => (
            <th key={item.key} style={item.style}>
              {renderHeaderItem(item.name)}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {compiledValue.map((item: any, index: number) => {
          const key = `row-${index}`;
          return (
            <tr key={key}>
              {header.map(headerItem => (
                <td key={headerItem.key}>{item[headerItem.key]}</td>
              ))}
            </tr>
          );
        })}
      </tbody>
    </Table>
  );
}
