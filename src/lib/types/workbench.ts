export type TodoItem = {
  id: string;
  text: string;
  done: boolean;
  createdAt: number;
  archived?: boolean;
  date?: string;
};

export type FilterType = "all" | "active" | "completed";
export type ModuleType = "todo" | "note";
export type NoteViewMode = "edit" | "preview" | "split";

export type NoteSummary = {
  groupId: string | null;
  id: string;
  title: string;
  fileName: string;
  updatedAt: number;
};

export type NoteDocument = {
  id: string;
  title: string;
  fileName: string;
  content: string;
  updatedAt: number;
  groupId?: string;
};

export type NoteGroup = {
  id: string;
  name: string;
  color?: string;
  createdAt: number;
  updatedAt: number;
};

export type NoteWithGroup = NoteSummary & {
  group?: NoteGroup;
};

export type GroupedNotes = {
  group: NoteGroup;
  notes: NoteSummary[];
}[];

export type StorageMode = "local" | "minio";

export type MinioConfig = {
  endpoint: string;
  bucket: string;
  accessKey: string;
  secretKey: string;
};

export type AppSettings = {
  storageMode: StorageMode;
  minio?: MinioConfig | null;
};

export type SyncResult = {
  uploaded: number;
  downloaded: number;
  skipped: number;
  errors: string[];
};
