import { ChangeDetectionStrategy, Component } from '@angular/core';

import { demoComponentContent } from './buttons-section.list';
import { ContentSection } from '../../common-docs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'buttons-section',
  templateUrl: './buttons-section.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ButtonsSectionComponent {
  name = 'Buttons';
  src = 'https://github.com/valor-software/ngx-bootstrap/tree/development/src/buttons';
  componentContent: ContentSection[] = demoComponentContent;
}
