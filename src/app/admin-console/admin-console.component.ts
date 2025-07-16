import { Component } from '@angular/core';
import { open, confirm } from '@tauri-apps/plugin-dialog';
import {invoke} from "@tauri-apps/api/core";

@Component({
  selector: 'app-admin-console',
  standalone: true,
  imports: [],
  templateUrl: './admin-console.component.html',
  styleUrl: './admin-console.component.css'
})
export class AdminConsoleComponent {
  course_name: string | null = null;
  course_date: string | null = null;
  async getFile(): Promise<void> {
    const file: string | null = await open({
      multiple: false,
      directory: false,
    });

    if(!file) {
        return
    }

    try {
      const file_data: string[] = await invoke('read_event', { filePath: file });
      this.course_name = file_data[0]
      this.course_date = file_data[1]
    } catch (err) {
      await confirm(
          `${err}`,
          {title: 'TEC - Errore Apertura File', kind: 'error'}
      );
    }

    try {
      await invoke('create_user', { username: this.course_name, expiration: this.course_date });
    }
    catch(err) {
      await confirm(
          `${err}`,
          {title: 'TEC - Errore', kind: 'error'}
      )
    }
  }


}
