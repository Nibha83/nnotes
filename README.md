# 📝 nnotes

`nnotes` is a fast and simple **command-line note-taking tool** built with Rust.  
It lets you quickly add, search, and delete notes right from your terminal.  

---

## ✨ Features

- Add notes with a title and content  
- Search notes instantly using [tantivy](https://github.com/quickwit-oss/tantivy) (full-text search)  
- Delete notes by ID  
- Notes are stored locally in JSON and indexed for fast searching  
- Cross-platform support (Linux, macOS, Windows)  

---

## 📦 Installation

### From source (requires Rust):
```bash
git clone git@github.com:Nibha83/nnotes.git
cd nnotes
cargo install --path .
````

### From crates.io (once published):

```bash
cargo install nnotes
```

---

## ⚡ Usage

```bash
nnotes "your search query"     # Search notes
nnotes "title" "content"       # Add a note
nnotes -d <NOTE_ID>            # Delete a note by ID
nnotes -h                      # Show help
```

### Examples

```bash
# Add a note
nnotes "Shopping List" "Eggs, Milk, Bread"

# Search notes
nnotes "Milk"

# Delete a note by ID
nnotes -d "534a0d71-9687-4432-a96b-8caf9284af8e"
```

---

## 📂 Data Storage

Your notes are stored locally at:

* **Linux/macOS**: `~/.local/share/nnotes/notes.json`
* **Windows**: `%APPDATA%\nnotes\notes.json`

The search index is also stored in the same folder.

---

## 🔑 Note IDs

Each note is automatically assigned a **UUID** (e.g. `534a0d71-9687-4432-a96b-8caf9284af8e`).
Use this ID to delete notes.

---

## 🛠 Development

Run the project locally:

```bash
cargo run -- "title" "content"
cargo run -- "search term"
cargo run -- -d <NOTE_ID>
```

---

## 📜 License

MIT License.
Feel free to use, modify, and share.

---

## 🙌 Contributing

PRs, issues, and suggestions are welcome!

