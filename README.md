# Ryver🌊
A Cli that generates luau/ts files from spreadsheets

## 📥Install

### With Aftman
Add an entry to the `[tools]` section of `aftman.toml`:
```toml
river = "piquu/ryver@0.1.0"
```

Or use the `aftman` Cli to add it:
```bash
aftman add piquu/ryver
```

### With cargo
```bash
cargo install ryver
```

## 📕Usage

### Arguments
* `-f`, `--file <FILE>`
    * Input file: `.xlsx`, `.xlsm`, `.xlsb`, `.xls`
* `-o`, `--out <FOLDER>`
    * Folder where the luau/ts files are put
* `-s`, `--sheet <SHEET>`
    * Sheets that luau/ts files should be generated for
* `--table-name <NUMBER>`
    * Spreadsheet column thats used as the table/object name
* `-n`, `--no-type`
    * Dont add `export type ...`/`export interface ...` to the luau/ts file
* `-t`, `--typescript`
    * Generate a ts file instead of a luau file

### Example
```bash
ryver -f example.xlsx -o ./out
```

```bash
ryver -f test.xlsx -o ./out -s Sheet1 -s Sheet2 --table-name 2 -t -n
```

## ✅Todo
* Add csv/tsv support
* Add Google Sheets support
* Add support for table/object names/entries with spaces

## 📋License
Ryver is licensed under the [MIT license](LICENSE).
