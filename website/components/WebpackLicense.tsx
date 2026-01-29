import { useLang } from '@rspress/core/runtime';
import { Link } from '@rspress/core/theme';
import type { FC } from 'react';

const WebpackLicense: FC<{ from: string | string[] }> = ({ from }) => {
  const links = Array.isArray(from) ? from : [from];
  const isEn = useLang() === 'en';

  const rootStyle = {
    fontSize: '14px',
  };

  const summaryStyle = {
    display: 'list-item',
    color: 'var(--rp-c-text-2)',
  };

  if (isEn) {
    return (
      <details style={rootStyle}>
        <summary style={summaryStyle}>CC 4.0 License</summary>
        <blockquote>
          <p>
            The content of this section is derived from the content of the
            following links and is subject to the CC BY 4.0 license.
          </p>
          <ul>
            {links.map((link) => (
              <li key={link}>
                <Link href={link}>{link}</Link>
              </li>
            ))}
          </ul>
          <p>
            The following contents can be assumed to be the result of
            modifications and deletions based on the original contents if not
            specifically stated.
          </p>
        </blockquote>
      </details>
    );
  }
  return (
    <details style={rootStyle}>
      <summary style={summaryStyle}>CC 4.0 协议</summary>
      <blockquote>
        <p>
          本节内容派生于以下链接指向的内容 ，并遵守 CC BY 4.0 许可证的规定。
        </p>
        <ul>
          {links.map((link) => (
            <li key={link}>
              <Link key={link} href={link}>
                {link}
              </Link>
            </li>
          ))}
        </ul>
        <p>
          以下内容如果没有特殊声明，可以认为都是基于原内容的修改和删减后的结果。
        </p>
      </blockquote>
    </details>
  );
};

export default WebpackLicense;
