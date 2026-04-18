import { invoke } from "@tauri-apps/api/core";

import type { TodoItem } from "$lib/types/workbench";

export function loadTodos() {
  return invoke<TodoItem[]>("load_todos");
}

export function saveTodos(todos: TodoItem[]) {
  return invoke("save_todos", { todos });
}
