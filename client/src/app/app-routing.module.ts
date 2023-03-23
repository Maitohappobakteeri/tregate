import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { TilemapComponent } from './tilemap/tilemap.component';
import { WorldComponent } from './world/world.component';
import { BuildingsComponent } from './buildings/buildings.component';

const routes: Routes = [
  { path: 'tilemap', component: TilemapComponent },
  { path: 'world', component: WorldComponent },
  { path: 'buildings', component: BuildingsComponent },
  { path: '', pathMatch: 'full', redirectTo: 'tilemap' },
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule],
})
export class AppRoutingModule {}
