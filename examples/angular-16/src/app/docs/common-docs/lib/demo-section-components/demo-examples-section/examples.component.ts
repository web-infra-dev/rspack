import { Component, HostListener } from '@angular/core';
import { ActivatedRoute, NavigationEnd, Router } from '@angular/router';
import sdk, { Project } from '@stackblitz/sdk';

import { ContentSection } from '../../models/content-section.model';
import { ComponentExample } from '../../models/components-examples.model';
import { main } from './stackblitz/main';
import { polyfills } from './stackblitz/polyfills';
import { getAppModuleCode, NgxModuleData } from './stackblitz/app.module';
import { getIndexHtmlCode } from './stackblitz/html';
import {
  getComponentClassName,
  getTagName,
  getTemplateFileName,
  getCSSCodeDatepickerCustomClass
} from './stackblitz/helpers';
import { Utils } from 'ngx-bootstrap/utils';
import { Subscription } from 'rxjs';
import { AvailableTabsNames } from '../../models/common.models';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'examples',
  templateUrl: './examples.component.html'
})
export class ExamplesComponent {
  examples: ComponentExample[];
  moduleData: NgxModuleData;
  tabName?: AvailableTabsNames;
  routeSubscription: Subscription;

  constructor(public section: ContentSection, private route: ActivatedRoute, router: Router) {
    this.examples = section.content as ComponentExample[];
    this.moduleData = this.route.snapshot.data && this.route.snapshot.data[1];
    this.moduleData.moduleRoute = router.routerState.snapshot.url;
    this.tabName = router.parseUrl(router.url).queryParams?.['tab'];
    this.routeSubscription = router.events.subscribe((event: any) => {
      if (event instanceof NavigationEnd) {
        this.tabName = router.parseUrl(router.url).queryParams?.['tab'];
      }
    });
  }

  @HostListener('document:click', ['$event'])
  preventEmptyHrefNav(event: MouseEvent & { target: Element }): void {
    let element: Element = event.target;
    let preventNav = element.getAttribute('href') === '#';

    if (preventNav) {
      event.preventDefault();

      return;
    }

    if (element.tagName !== 'A') {
      while (element.parentElement && element !== document.body) {
        if (preventNav) {
          event.preventDefault();

          return;
        }
        element = element.parentElement;
        preventNav = element.getAttribute('href') === '#';
      }
    }
  }

  openStackBlitzDemo(ts?: string, html?: string) {
    if (!ts || !html) {
      return;
    }

    const className = getComponentClassName(ts);
    const tag = getTagName(ts);
    const templateName = getTemplateFileName(ts);
    if (tag && className) {
      const project: Project = {
        template: 'angular-cli',
        title: `ngx-bootstrap stackblitz demo `,
        description: 'stackblitz demo',
        files: {
          'index.html': getIndexHtmlCode(tag, this.moduleData, Utils.stackOverflowConfig()),
          'styles.css': `body {padding: 30px; position: relative}
        ${this.moduleData.moduleRoute === '/sortable' ?
            `.sortable-item {
      padding: 6px 12px;
      margin-bottom: 4px;
      font-size: 14px;
      line-height: 1.4em;
      text-align: center;
      cursor: grab;
      border: 1px solid transparent;
      border-radius: 4px;
      border-color: #adadad;
    }

    .sortable-item-active {
      background-color: #e6e6e6;
      box-shadow: inset 0 3px 5px rgba(0,0,0,.125);
    }

    .sortable-wrapper {
      min-height: 150px;
    }` : ''}
    ${this.moduleData.moduleRoute === '/accordion' ?
            `.card.customClass,
.card.customClass .card-header,
.panel.customClass {
  background-color: #5bc0de;
  color: #fff;
}
.panel.customClass .panel-body {
  background-color: #337aa7;
}` : ''}`,
          '.angular-cli.json': `{"apps": [{"styles": ["styles.css"]}]}`,
          'main.ts': main,
          'polyfills.ts': polyfills,
          'app/app.module.ts': getAppModuleCode(className, this.moduleData),
          'app/ngx-bootstrap-demo.component.ts': this.getTs(ts)
        },
        dependencies: {
          '@angular/animations': 'latest',
          'web-animations-js': 'latest',
          'ngx-bootstrap': 'next'
        },

      };
      if (className === 'DemoDatepickerDateCustomClassesComponent') {
        project.files['app/date-custom-classes.scss'] = getCSSCodeDatepickerCustomClass();
      }
      project.files[`app/${templateName}`] = this.getHtml(html);

      sdk.openProject(project);
    }
  }

  initFragment(anchor: string) {
    const spAnchor = anchor.split('-');
    return spAnchor.slice(0, spAnchor.length - 1).join('-');
  }

  private getHtml(html: string): string {
    return this.moduleData.moduleRoute === '/carousel' ?
      html.replace(/src="/g, 'src="https://valor-software.com/ngx-bootstrap/') : html;
  }

  private getTs(ts: string): string {
    return this.moduleData.moduleRoute === '/carousel' ?
      ts.replace(/assets/g, 'https://valor-software.com/ngx-bootstrap/assets') : ts;
  }
}

