use rocksdb::{Options, DB};


// Currently we use rocksdb. Need evaluate if we should switch to lmdb for a better read performance.
pub fn init_db(path: &str) -> DB{
    let mut opts = Options::default();
    opts.create_if_missing(true);
    DB::open(&opts, path).expect(&format!("failed to open db at path {}", path))
}