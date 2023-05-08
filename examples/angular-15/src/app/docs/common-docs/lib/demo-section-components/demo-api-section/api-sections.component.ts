import { ChangeDetectionStrategy, Component, Injector } from '@angular/core';

import { ContentSection } from '../../models/content-section.model';
import { ComponentApi } from '../../models/components-api.model';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'api-sections',
  templateUrl: './api-sections.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ApiSectionsComponent {
  apiSectionsContent: ComponentApi[];
  _injectors = new Map<ComponentApi, Injector>();

  constructor(public section: ContentSection, private injector: Injector) {
    this.apiSectionsContent = section.content as ComponentApi[];
  }

  sectionInjections(_content: ComponentApi): Injector {
    if (this._injectors.has(_content)) {
      return this._injectors.get(_content) as Injector;
    }

    const _injector = Injector.create([{
      provide: ComponentApi,
      useValue: _content
    }], this.injector);

    this._injectors.set(_content, _injector);

    return _injector;
  }
}
