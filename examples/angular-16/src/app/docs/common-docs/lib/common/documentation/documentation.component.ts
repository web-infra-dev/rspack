import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'documentation',
  templateUrl: './documentation.component.html'
})
export class DocumentationComponent {
  name = `Native Angular widgets for Bootstrap 5 and Bootstrap 4`;
  src = 'https://github.com/valor-software/ngx-bootstrap';
}
