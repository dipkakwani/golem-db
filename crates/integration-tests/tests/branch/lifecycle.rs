use golemdb_branch::{BranchError, BranchId, Branches, OperationError};
use golemdb_cells::{CellKey, CellNameRef, CellValue, tables};
use golemdb_merkle::{HashProvider, Keccak256Hasher};
use golemdb_storage::{Database, MemoryDatabase, ReadTransaction, Table, WriteTransaction};

const SUPERBLOCK: Table = Table("Superblock");

fn key() -> CellKey {
    CellKey::new(64, CellNameRef::raw(b"name"))
}

fn value(text: &str) -> CellValue {
    CellValue::parse([b"\x02".as_slice(), text.as_bytes()].concat()).unwrap()
}

// Simulate external publication independently of the branch reader under test.
// Placeholder roots suffice here: these tests do not seal or commit.
fn publish(db: &impl Database, commit: u64, text: &str) {
    let mut row = commit.to_be_bytes().to_vec();
    row.extend_from_slice(&[0x11; 32]);
    row.extend_from_slice(&[0x22; 32]);
    let mut tx = db.begin_write().unwrap();
    tx.put(SUPERBLOCK, b"head", &row).unwrap();
    tx.put(tables::CELL, &key().encode(), value(text).encoded_bytes())
        .unwrap();
    tx.commit().unwrap();
}

fn get<D: Database>(
    branches: &Branches<D, impl HashProvider>,
    handle: BranchId,
) -> Option<CellValue> {
    branches.read(handle, |cells| cells.get(&key())).unwrap()
}

fn put<D: Database>(branches: &Branches<D, impl HashProvider>, handle: BranchId, text: &str) {
    branches
        .write(handle, |cells| {
            cells.put(key(), value(text));
            Ok::<_, BranchError>(())
        })
        .unwrap();
}

fn assert_invalid<D: Database>(branches: &Branches<D, impl HashProvider>, handle: BranchId) {
    assert!(matches!(
        branches.branch_info(handle),
        Err(BranchError::HandleInvalid)
    ));
    // Invalid calls must never invoke user callbacks, even for overlay hits.
    assert!(matches!(
        branches.read(handle, |_| -> Result<(), ()> {
            panic!("invalid read admitted")
        }),
        Err(OperationError::Branch(BranchError::HandleInvalid))
    ));
    assert!(matches!(
        branches.write(handle, |_| -> Result<(), ()> {
            panic!("invalid write admitted")
        }),
        Err(OperationError::Branch(BranchError::HandleInvalid))
    ));
    assert!(matches!(
        branches.checkpoint(handle),
        Err(BranchError::HandleInvalid)
    ));
    assert!(matches!(
        branches.rollback(handle),
        Err(BranchError::HandleInvalid)
    ));
    assert!(matches!(
        branches.discard(handle),
        Err(BranchError::HandleInvalid)
    ));
}

fn lifecycle(db: impl Database + Clone) {
    publish(&db, 7, "origin");
    let branches = Branches::new(db.clone(), Keccak256Hasher).unwrap();
    assert_eq!(branches.head().unwrap(), 7);
    let a = branches.begin().unwrap();
    let b = branches.begin().unwrap();
    assert_eq!(branches.branch_info(a).unwrap().commit_id, 7);
    assert_eq!(branches.branch_info(b).unwrap().commit_id, 7);
    assert!(b > a);
    put(&branches, a, "one");
    branches.checkpoint(a).unwrap();
    put(&branches, a, "two");
    assert_eq!(get(&branches, a), Some(value("two")));
    assert_eq!(get(&branches, b), Some(value("origin")));
    branches.rollback(a).unwrap();
    assert_eq!(get(&branches, a), Some(value("one")));
    branches.rollback(a).unwrap();
    assert_eq!(get(&branches, a), Some(value("origin")));
    assert!(matches!(
        branches.rollback(a),
        Err(BranchError::NoFrameToRollback)
    ));
    // Record scans also stay inside the validated callback/snapshot.
    let rows = branches
        .read(a, |cells| {
            cells
                .scan_prefix(&64u64.to_be_bytes())?
                .collect::<golemdb_branch::Result<Vec<_>>>()
        })
        .unwrap();
    assert_eq!(rows, vec![(key(), value("origin"))]);
    put(&branches, a, "staged");
    assert_eq!(
        db.begin_read()
            .unwrap()
            .get(tables::CELL, &key().encode())
            .unwrap(),
        Some(value("origin").into_bytes())
    );
    branches.discard(a).unwrap();
    assert_invalid(&branches, a);
    assert_eq!(get(&branches, b), Some(value("origin")));
    let c = branches.begin().unwrap();
    assert!(c > b);
    assert_eq!(get(&branches, c), Some(value("origin")));

    // An advance invalidates both branches, including reads of staged cells.
    put(&branches, b, "must not escape");
    publish(&db, 8, "new head");
    assert_eq!(branches.head().unwrap(), 8);
    assert_invalid(&branches, b);
    assert_invalid(&branches, c);
    let next = branches.begin().unwrap();
    assert_eq!(branches.branch_info(next).unwrap().commit_id, 8);
    assert_eq!(get(&branches, next), Some(value("new head")));
    // Root and head rows were not overwritten by any branch lifecycle call.
    let head = db
        .begin_read()
        .unwrap()
        .get(SUPERBLOCK, b"head")
        .unwrap()
        .unwrap();
    assert_eq!(&head[..8], &8u64.to_be_bytes());
    assert_eq!(&head[8..40], &[0x11; 32]);
    assert_eq!(&head[40..], &[0x22; 32]);
}

#[test]
fn memory_lifecycle() {
    lifecycle(MemoryDatabase::new());
}

#[test]
fn mdbx_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    lifecycle(golemdb_storage_mdbx::MdbxDatabase::open(dir.path()).unwrap());
}
