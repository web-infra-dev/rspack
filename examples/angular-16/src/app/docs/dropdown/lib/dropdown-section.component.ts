import { ChangeDetectionStrategy, Component } from '@angular/core';

import { demoComponentContent } from './dropdown-section.list';
import { ContentSection } from '../../common-docs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'dropdown-section',
  templateUrl: './dropdown-section.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class DropdownSectionComponent {
  name = 'Dropdowns';
  src = 'https://github.com/valor-software/ngx-bootstrap/tree/development/src/dropdown';
  componentContent: ContentSection[] = demoComponentContent;
}
