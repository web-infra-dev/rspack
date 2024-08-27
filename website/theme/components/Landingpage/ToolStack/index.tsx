import type React from 'react';
import { memo } from 'react';
import { useLang } from 'rspress/runtime';
import { Link } from 'rspress/theme';
import { useI18n } from '../../../i18n';
import sharedStyles from '../shared.module.scss';
import styles from './index.module.scss';

const ToolStack: React.FC = memo(() => {
  const lang = useLang();
  const t = useI18n();
  const isEn = lang === 'en';
  const tools = [
    {
      name: 'Rspack',
      desc: isEn
        ? 'A high performance JavaScript bundler written in Rust, with a webpack-compatible API.'
        : '基于 Rust 编写的高性能 JavaScript 打包工具，具备与 webpack 兼容的 API。',
      logo: 'https://assets.rspack.dev/rspack/rspack-logo.svg',
      url: 'https://rspack.dev',
    },
    {
      name: 'Rsbuild',
      desc: isEn
        ? 'An Rspack-based build tool that provides out-of-the-box setup for enjoyable development experience.'
        : '基于 Rspack 的构建工具，包含开箱即用的预设配置，带来愉悦的开发体验。',
      logo: 'https://assets.rspack.dev/rsbuild/rsbuild-logo.svg',
      url: 'https://rsbuild.dev',
    },
    {
      name: 'Rspress',
      desc: isEn
        ? 'A static site generator based on Rsbuild and MDX for creating elegant documentation sites.'
        : '基于 Rsbuild 和 MDX 的静态站点生成器，用于创建优雅的文档站点。',
      logo: 'https://assets.rspack.dev/rspress/rspress-logo-480x480.png',
      url: 'https://rspress.dev',
    },
    {
      name: 'Rsdoctor',
      desc: isEn
        ? 'A powerful one-stop build analyzer for visualizing the build process and build artifacts.'
        : '强大的一站式构建分析工具，用于可视化构建过程和构建产物。',
      logo: 'https://assets.rspack.dev/rsdoctor/rsdoctor-logo-480x480.png',
      url: 'https://rsdoctor.dev',
    },

    {
      name: 'Rslib',
      desc: isEn
        ? 'A library build tool powered by Rsbuild for developing libraries or UI components.'
        : '基于 Rsbuild 的库构建工具，用于开发工具库或 UI 组件库。',
      logo: 'https://assets.rspack.dev/rslib/rslib-logo.svg',
      url: 'https://github.com/web-infra-dev/rslib',
    },
  ];

  return (
    <section className={sharedStyles.container}>
      <div className={sharedStyles.innerContainer}>
        <div className={sharedStyles.titleAndDesc}>
          <h2 className={sharedStyles.title}>{t('toolStackTitle')}</h2>
          <p className={sharedStyles.desc}>{t('toolStackDesc')}</p>
        </div>
        <div className={styles.tools}>
          {tools.map(({ name, desc, logo, url }) => {
            return (
              <Link className={styles.tool} key={name} href={url}>
                <img src={logo} alt={name} className={styles.logo} />
                <h3 className={styles.toolTitle}>{name}</h3>
                <p className={styles.toolDescription}>{desc}</p>
              </Link>
            );
          })}
        </div>
      </div>
    </section>
  );
});

export default ToolStack;
