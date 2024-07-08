import type { FC } from 'react';
type DefaultValue = {
  defaultValue: string;
  mode?: 'development' | 'production' | 'none';
};
const PropertyType: FC<{ type: string; defaultValueList?: DefaultValue[] }> & {
  CN: FC<{ type: string; defaultValueList?: DefaultValue[] }>;
} = ({ type, defaultValueList }) => {
  return (
    <ul className="list-disc pl-5 my-4 leading-7">
      <li className="[&:not(:first-child)]:mt-2">
        <strong>Type:</strong> <code>{type}</code>
      </li>
      {defaultValueList?.length && defaultValueList.length > 0 && (
        <li className="[&:not(:first-child)]:mt-2">
          <strong>Default: </strong>
          {defaultValueList.map(({ defaultValue, mode }, index) => {
            return (
              <span key={defaultValue}>
                {index > 0 && <span>, </span>}
                {mode && (
                  <>
                    <a
                      style={{ marginLeft: '4px' }}
                      href={`/config/mode#${mode}`}
                    >
                      {mode} mode
                    </a>{' '}
                    <span>is</span>
                  </>
                )}
                <code style={{ marginLeft: '4px' }}>{defaultValue}</code>
              </span>
            );
          })}
        </li>
      )}
    </ul>
  );
};
PropertyType.CN = ({ type, defaultValueList }) => {
  return (
    <ul className="list-disc pl-5 my-4 leading-7">
      <li className="[&:not(:first-child)]:mt-2">
        <strong>类型：</strong> <code>{type}</code>
      </li>
      {defaultValueList?.length && defaultValueList.length > 0 && (
        <li className="[&:not(:first-child)]:mt-2">
          <strong>默认值: </strong>
          {defaultValueList.map(({ defaultValue, mode }, index) => {
            return (
              <span key={defaultValue}>
                {index > 0 && <span>, </span>}
                {mode && (
                  <>
                    <a
                      style={{ marginLeft: '4px' }}
                      href={`/config/mode#${mode}`}
                    >
                      {mode} 模式
                    </a>{' '}
                    <span>为</span>
                  </>
                )}
                <code style={{ marginLeft: '4px' }}>{defaultValue}</code>
              </span>
            );
          })}
        </li>
      )}
    </ul>
  );
};

export default PropertyType;
