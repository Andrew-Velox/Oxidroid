use std::{fs, path::PathBuf};

#[derive(Clone)]
pub struct FileEntry { pub name: String, pub path: PathBuf, pub is_dir: bool, pub size: u64, pub count: usize }

pub struct FileExplorer { pub current_path: PathBuf, pub items: Vec<FileEntry>, pub selected: usize, pub offset: usize, pub focused: bool }

impl FileExplorer {
    pub fn new(start: &str) -> Self {
        let mut fe = Self { current_path: PathBuf::from(start), items: Vec::new(), selected: 0, offset: 0, focused: false };
        fe.refresh();
        fe
    }
    pub fn refresh(&mut self) {
        self.items.clear();
        self.items.push(FileEntry { name: "..".into(), path: self.current_path.parent().unwrap_or(&self.current_path).to_path_buf(), is_dir: true, size: 0, count: 0 });
        if let Ok(rd) = fs::read_dir(&self.current_path) {
            let mut entries: Vec<FileEntry> = rd.flatten().filter_map(|e| {
                let path = e.path();
                let name = path.file_name()?.to_string_lossy().into_owned();
                let is_dir = path.is_dir();
                let (size, count) = if is_dir { (0, fs::read_dir(&path).map(|d| d.count()).unwrap_or(0)) } else { (fs::metadata(&path).map(|m| m.len()).unwrap_or(0), 0) };
                Some(FileEntry { name, path, is_dir, size, count })
            }).collect();
            entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
            self.items.extend(entries);
        }
    }
    pub fn up(&mut self) { if self.selected > 0 { self.selected -= 1; } }
    pub fn down(&mut self) { if self.selected + 1 < self.items.len() { self.selected += 1; } }
    pub fn enter(&mut self) { if let Some(e) = self.items.get(self.selected) { if e.is_dir { self.current_path = e.path.clone(); self.selected = 0; self.offset = 0; self.refresh(); } } }
    pub fn visible<'a>(&'a mut self, max: usize) -> &'a [FileEntry] {
        if self.selected < self.offset { self.offset = self.selected; } else if self.selected >= self.offset + max { self.offset = self.selected + 1 - max; }
        let end = (self.offset + max).min(self.items.len());
        &self.items[self.offset..end]
    }
}