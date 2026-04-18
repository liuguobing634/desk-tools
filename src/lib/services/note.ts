import { invoke } from "@tauri-apps/api/core";
import { marked } from "marked";

import type { NoteDocument, NoteSummary, NoteGroup } from "$lib/types/workbench";

marked.setOptions({
  breaks: true,
  gfm: true
});

export function listNotes() {
  return invoke<NoteSummary[]>("list_notes");
}

export function loadNote(id: string) {
  return invoke<NoteDocument>("load_note", { id });
}

export function createNote(title: string, groupId?: string) {
  return invoke<NoteDocument>("create_note", { 
    title,
    groupId: groupId || undefined
  });
}

export function saveNote(id: string, content: string) {
  return invoke<NoteDocument>("save_note", { id, content });
}

export function deleteNote(id: string) {
  return invoke("delete_note", { id });
}

export function renderMarkdown(content: string) {
  return marked.parse(content || "> 还没有内容，开始记录第一条 Markdown 笔记吧。") as string;
}

// 分组相关函数
export function listNoteGroups() {
  return invoke<NoteGroup[]>("list_note_groups");
}

export function createNoteGroup(name: string, color?: string) {
  return invoke<NoteGroup>("create_note_group", { name, color });
}

export function updateNoteGroup(id: string, name: string, color?: string) {
  return invoke<NoteGroup>("update_note_group", { id, name, color });
}

export function deleteNoteGroup(id: string) {
  return invoke("delete_note_group", { id });
}

// 移动笔记到分组
export function moveNoteToGroup(noteId: string, groupId: string | null) {
  return invoke<NoteDocument>("move_note_to_group", { 
    noteId, 
    groupId: groupId || undefined 
  });
}

// 导入图片
export function importImage() {
  return invoke<string | null>("import_image");
}
