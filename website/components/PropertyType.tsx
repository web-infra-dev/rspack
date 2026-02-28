import { useLang } from '@rspress/core/runtime';
import { Link } from '@rspress/core/theme';
import type { FC } from 'react';

type DefaultValue = {
  defaultValue: string;
  mode?: 'development' | 'production' | 'none';
};

const PropertyType: FC<{ type: string; defaultValueList?: DefaultValue[] }> = ({
  type,
  defaultValueList,
}) => {
  const lang = useLang();
  return (
    <ul className="list-disc pl-5 my-4 leading-7">
      <li className="[&:not(:first-child)]:mt-2">
        <span className="font-semibold">
          {lang === 'zh' ? '类型：' : 'Type:'}
        </span>{' '}
        <code>{type}</code>
      </li>
      {defaultValueList?.length && defaultValueList.length > 0 && (
        <li className="[&:not(:first-child)]:mt-2">
          <span className="font-semibold">
            {lang === 'zh' ? '默认值：' : 'Default:'}
          </span>
          {defaultValueList.map(({ defaultValue, mode }, index) => {
            return (
              <span key={defaultValue}>
                {index > 0 && <span>, </span>}
                {mode && (
                  <>
                    <Link
                      style={{ marginLeft: '4px' }}
                      href={`/config/mode#${mode}`}
                    >
                      {mode} {lang === 'zh' ? '模式' : 'mode'}
                    </Link>
                    &nbsp;
                    <span>{lang === 'zh' ? '为' : 'is'}</span>
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
