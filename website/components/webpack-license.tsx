import { FC } from 'react';
import { useLang } from 'rspress/runtime';

const WebpackLicense: FC<{ from: string | string[] }> = ({ from }) => {
  const links = Array.isArray(from) ? from : [from];
  const isEn = useLang() == 'en';
  if (isEn) {
    return (
      <summary>
        <small>CC 4.0 License</small>

        <details>
          <blockquote>
            <p>
              The content of this section is derived from the content of the
              following links and is subject to the CC BY 4.0 license.
            </p>
            <ul>
              {links.map(link => (
                <li key={link}>
                  <a href={link}>{link}</a>
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
      </summary>
    );
  } else {
    return (
      <summary>
        <small>CC 4.0 协议声明</small>

        <details>
          <blockquote>
            <p>
              本节内容派生于以下链接指向的内容 ，并遵守 CC BY 4.0 许可证的规定。
            </p>
            <ul>
              {links.map(link => (
                <li key={link}>
                  <a key={link} href={link}>
                    {link}
                  </a>
                </li>
              ))}
            </ul>
            <p>
              以下内容如果没有特殊声明，可以认为都是基于原内容的修改和删减后的结果。
            </p>
          </blockquote>
        </details>
      </summary>
    );
  }
};

export default WebpackLicense;
