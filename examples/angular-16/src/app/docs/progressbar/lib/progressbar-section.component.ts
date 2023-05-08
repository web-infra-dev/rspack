import { ChangeDetectionStrategy, Component, Injector } from '@angular/core';
import { demoComponentContent } from './progressbar-section.list';
import { ContentSection } from '../../common-docs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'progressbar-section',
  templateUrl: './progressbar-section.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ProgressbarSectionComponent {
  name = 'Progressbar';
  src = 'https://github.com/valor-software/ngx-bootstrap/tree/development/src/progressbar';
  componentContent: ContentSection[] = demoComponentContent;
  content: any;

  _injectors = new Map<ContentSection, Injector>();

  constructor(private injector: Injector) { }

  sectionInjections(content: ContentSection) {
    if (this._injectors.has(content)) {
      return this._injectors.get(content);
    }

    const _injector = Injector.create([
      {
        provide: ContentSection,
        useValue: content
      }], this.injector);

    this._injectors.set(content, _injector);

    return _injector;
  }
}
