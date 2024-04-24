import type { FC } from 'react';
type DefaultValue = {
  defaultValue: string;
  mode?: 'development' | 'production' | 'none';
};
const PropertyType: FC<{ type: string; defaultValueList?: DefaultValue[] }> & {
  CN: FC<{ type: string; defaultValueList?: DefaultValue[] }>;
} = ({ type, defaultValueList }) => {
  return (
    <ul>
      <li style={{ marginBottom: '10px' }}>
        <strong>Type:</strong> <code>{type}</code>
      </li>
      {defaultValueList?.length && defaultValueList.length > 0 && (
        <li>
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
    <ul>
      <li style={{ marginBottom: '10px' }}>
        <strong>类型：</strong> <code>{type}</code>
      </li>
      {defaultValueList?.length && defaultValueList.length > 0 && (
        <li>
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
