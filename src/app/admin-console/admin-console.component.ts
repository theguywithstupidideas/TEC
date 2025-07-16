import {Component, OnInit} from '@angular/core';
import { open, confirm } from '@tauri-apps/plugin-dialog';
import {invoke} from "@tauri-apps/api/core";
import {NgForOf, NgIf} from "@angular/common";
import {AccountButtonComponent} from "../UI/account-button/account-button.component";

@Component({
  selector: 'app-admin-console',
  standalone: true,
  imports: [
    NgIf,
    AccountButtonComponent,
    NgForOf
  ],
  templateUrl: './admin-console.component.html',
  styleUrl: './admin-console.component.css'
})
export class AdminConsoleComponent implements OnInit{
  course_name: string | null = null;
  course_date: string | null = null;
  users: string[] | null = null;

  async ngOnInit() {
    try{
      await this.getExpiredUsers()
    }
    catch(error){
      await confirm(
          `${error}`,
          {title: 'TEC - Errore Apertura File', kind: 'error'}
      )
    }
  }

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
      await confirm(
          'Corso addiunto.',
          {title: 'TEC', kind: 'info'}
      )
    }
    catch(err) {
      await confirm(
          `${err}`,
          {title: 'TEC - Errore', kind: 'error'}
      )
    }
  }

  async getExpiredUsers(): Promise<void> {
    try {
      this.users = await invoke('clean_up');
    }
    catch(err) {
      await confirm(
          `${err}`,
          {title: 'TEC - Errore Apertura File', kind: 'error'}
      )
    }
  }
}
