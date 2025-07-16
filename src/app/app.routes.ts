import { Routes } from "@angular/router";
import { AdminConsoleComponent } from "./admin-console/admin-console.component";
import { AppComponent } from "./app.component";
import {HomeComponent} from "./home/home.component";

export const routes: Routes = [
    { path: '', component: HomeComponent, pathMatch: 'full' },
    { path: 'admin-console', component: AdminConsoleComponent }
];
