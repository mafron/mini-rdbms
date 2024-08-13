use std::fs::{File, OpenOptions}; // File構造体はファイルディスクリプタのラッパー
use std::path::Path;
use std::io::{self, prelude::*, SeekFrom};

// ページサイズ：4096Byte固定
const PAGE_SIZE: u64 = 4096;

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
        let next_page_id = file_size / PAGE_SIZE;

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
        let offset = PAGE_SIZE * page_id.to_u64();
        self.heap_file.seek(SeekFrom::Start(offset))?;

        // データの読み込み
        self.heap_file.read_exact(data)
    }

    // データの書き込み
    pub fn write(&mut self, page_id: PageID, data: &[u8]) -> io::Result<()> {
        let offset = PAGE_SIZE * page_id.to_u64();
        self.heap_file.seek(SeekFrom::Start(offset))?;

        // データの書き込み
        self.heap_file.write_all(data)
    }
}