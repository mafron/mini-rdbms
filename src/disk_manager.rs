use std::fs::{File, OpenOptions}; // File構造体はファイルディスクリプタのラッパー
use std::path::Path;
use std::io::{self, prelude::*, SeekFrom};

// ページサイズ：4096Byte固定
const PAGE_SIZE: usize = 4096;

// ディスクマネージャ
// ・ディスクへのファイル（ヒープファイル）の読み書きを行う
// ・ヒープファイルはページ（固定長ブロック、大体は4Byte、OSのファイルシステムの読み書きサイズに合わせている）で構成され、ページ単位で読み書きを実施
// ・ページIDの採番によりヒープファイルにページを作成
pub struct DiskManager {
    // ヒープファイルのファイルディスクリプタ
    heap_file: File,
    // 次に採番するページID（0始まり）
    // 採番のたびにインクリメント
    next_page_id: u64,
}

// ページID（NewTypeパターン）
#[derive(Clone, Copy, Debug)]
pub struct PageID(pub u64);

impl PageID {
    pub fn to_u64(self) -> u64 {
        self.0
    }
}

impl DiskManager {
    // コンストラクタ
    pub fn new(heap_file_path: impl AsRef<Path>) -> io::Result<Self> {
        // ヒープファイルのオープン
        let heap_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(heap_file_path)?;

        // ファイルサイズの取得から次に採番するページIDを計算 
        let file_size = heap_file.metadata()?.len();
        let next_page_id = file_size / PAGE_SIZE as u64;

        Ok(Self {
            heap_file,
            next_page_id,
        })
    }

    // ページの割り当て
    pub fn allocate_page(&mut self) -> PageID {
        let page_id = self.next_page_id;
        self.next_page_id += 1;
        PageID(page_id)
    }

    // データの読み込み
    pub fn read(&mut self, page_id: PageID, data: &mut [u8]) -> io::Result<()> {
        // ファイルディスクリプタを読み込むデータの先頭にシーク
        let offset = PAGE_SIZE as u64 * page_id.to_u64();
        self.heap_file.seek(SeekFrom::Start(offset))?;

        // データの読み込み
        self.heap_file.read_exact(data)
    }

    // データの書き込み
    pub fn write(&mut self, page_id: PageID, data: &[u8]) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * page_id.to_u64();
        self.heap_file.seek(SeekFrom::Start(offset))?;

        // データの書き込み
        self.heap_file.write_all(data)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test() {
        let (_, data_file_path) = NamedTempFile::new().unwrap().into_parts();
        let mut disk = DiskManager::new(&data_file_path).unwrap();

        // PageIDの採番
        let test_page_id = disk.allocate_page();

        // tempファイルへの書き込み
        let mut data = Vec::with_capacity(PAGE_SIZE); // ページサイズのベクターを確保
        data.extend_from_slice(b"test"); // 文字列の格納
        data.resize(PAGE_SIZE, 0); // 0-padding

        disk.write(test_page_id, &data).unwrap();

        let second_test_page_id = disk.allocate_page();
        let mut data2 = Vec::with_capacity(PAGE_SIZE);
        data2.extend_from_slice(b"second_test");
        data2.resize(PAGE_SIZE, 0);

        disk.write(second_test_page_id, &data2).unwrap();

        drop(disk);

        // tempファイルの読み込み
        let mut disk2 = DiskManager::new(data_file_path).unwrap();

        let mut buffer = vec![0; PAGE_SIZE];
        disk2.read(test_page_id, &mut buffer);
        assert_eq!(data, buffer);
        disk2.read(second_test_page_id, &mut buffer);
        assert_eq!(data2, buffer);
    }
}