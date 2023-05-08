export interface NgxModuleData {
  moduleName: string;
  moduleFolder: string;
  moduleRoute: string;
}


export function getAppModuleCode(className: string, moduleData: NgxModuleData) {
  return `import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import { ${moduleData.moduleName} } from 'ngx-bootstrap/${moduleData.moduleFolder}';
${className === 'DemoModalWithPopupsComponent' ? `import { TooltipModule } from 'ngx-bootstrap/tooltip';
import { PopoverModule } from 'ngx-bootstrap/popover';` : ''}
${className === 'DemoDatepickerChangeLocaleComponent' ? `import { defineLocale } from 'ngx-bootstrap/chronos';
import { deLocale, frLocale, plLocale } from 'ngx-bootstrap/locale';
 defineLocale('de', deLocale);
 defineLocale('fr', frLocale);
 defineLocale('pl', plLocale);` : ''}

import { ${className === 'DemoModalServiceFromComponent' ? `${className}, ModalContentComponent` : className} } from './ngx-bootstrap-demo.component'
@NgModule({
  declarations: [${className === 'DemoModalServiceFromComponent' ? `${className}, ModalContentComponent` : className}],
  imports: [
    ${moduleData.moduleName}.forRoot(),
    ${className === 'DemoModalWithPopupsComponent' ? `TooltipModule.forRoot(),
    PopoverModule.forRoot(),` : ''}
    BrowserAnimationsModule,
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    BrowserModule
  ],
  entryComponents: [${className === 'DemoModalServiceFromComponent' ? 'ModalContentComponent' : ''}],
  bootstrap: [${className}]
})
export class AppModule {
}
`;
}
