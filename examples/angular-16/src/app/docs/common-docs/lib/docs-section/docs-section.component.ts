import { ContentSection } from '../models/content-section.model';
import {
  ChangeDetectorRef,
  Component,
  Injector,
  Input,
  OnDestroy
} from "@angular/core";
import { ActivatedRoute, NavigationEnd, Router, NavigationExtras } from "@angular/router";
import { Subscription } from "rxjs";

const availableTabsPaths = ['overview', 'api', 'examples'] as const;
type AvailableTabsPathsType = (typeof availableTabsPaths)[number];

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'docs-section',
  template: `
    <ng-container *ngIf="content">
      <tabset class="example-tabset-box">
        <tab heading="Overview" [customClass]="'example-tabset'" [active]="overview" (selectTab)="onSelect('overview')">
          <ng-container *ngComponentOutlet="content[0].outlet; injector: sectionInjections(content[0])"></ng-container>
        </tab>
        <tab heading="API" [customClass]="'example-tabset'" [active]="api" (selectTab)="onSelect('api')">
          <ng-container *ngComponentOutlet="content[1].outlet; injector: sectionInjections(content[1])"></ng-container>
        </tab>
        <tab heading="Examples" [customClass]="'example-tabset'" [active]="examples" (selectTab)="onSelect('examples')">
          <ng-container *ngComponentOutlet="content[2].outlet; injector: sectionInjections(content[2])"></ng-container>
        </tab>
      </tabset>
      <add-nav class="add-nav" [componentContent]="overview ? content[0] : api ? content[1] : content[2]"></add-nav>
    </ng-container>
  `
})
export class DocsSectionComponent implements OnDestroy {
  @Input() content: ContentSection[] | undefined;
  _injectors = new Map<ContentSection, Injector>();
  routeSubscription: Subscription;
  overview = false;
  api = false;
  examples = false;

  constructor(
    private injector: Injector,
    private activatedRoute: ActivatedRoute,
    private router: Router,
    private changeDetection: ChangeDetectorRef
  ) {
    this.routeSubscription = this.router.events.subscribe((event: any) => {
      if (event instanceof NavigationEnd) {
        const params = this.router.parseUrl(event.url).queryParams;
        this.initActiveTab(params["tab"]?.toString());
      }
    });
  }

  initActiveTab(activeTab?: string) {
    this.resetTabs();
    if (!activeTab || !this.checkActiveTab(activeTab)) {
      this.overview = true;
      this.onSelect('overview');
      return;
    }

    this[activeTab as AvailableTabsPathsType] = true;
    this.changeDetection.detectChanges();
  }

  checkActiveTab(activeTab: string): boolean {
    return activeTab === 'overview' || activeTab === 'api' || activeTab === 'examples';
  }

  onSelect(tabName: string) {
    this.resetTabs();
    const extras: NavigationExtras = {queryParams: {tab: tabName}, fragment: this.router.parseUrl(this.router.url).fragment || undefined};
    this.router.navigate([], extras);
    this[tabName as AvailableTabsPathsType] = true;
  }

  sectionInjections(_content: ContentSection): Injector {
    if (this._injectors.has(_content)) {
      return this._injectors.get(_content) as Injector;
    }

    const _injector = Injector.create([{
      provide: ContentSection,
      useValue: _content
    }], this.injector);
    this._injectors.set(_content, _injector);
    return _injector;
  }

  resetTabs() {
    this.overview = false;
    this.api = false;
    this.examples = false;
  }

  ngOnDestroy() {
    this.routeSubscription.unsubscribe();
  }
}
