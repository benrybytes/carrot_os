
pub const DIRECT_POINTERS: u64 = 12;

pub struct SuperBlock {
    pub magic: u32,
    pub block_size: u32,
    pub created_at: u64,
    pub modified_at: Option<u64>,
    pub last_mounted_at: Option<u64>,
    pub block_count: u32,
    pub inode_count: u32,
    pub free_blocks: u32,
    pub free_inodes: u32,
    pub groups: u32,
    pub data_blocks_per_group: u32,
    pub uid: u32,
    pub gid: u32,
    pub checksum: u32,
}

pub struct INode {
    pub mode: u32, 
    pub hard_links: u16,
    pub user_id: u32,
    pub group_id: u32,
    pub block_count: u32, // should be in 512 bytes blocks
    pub size: u64,
    pub created_at: u64,
    pub accessed_at: Option<i64>,
    pub modified_at: Option<i64>,
    pub changed_at: Option<i64>,
    pub direct_blocks: [u32; DIRECT_POINTERS as usize],
    pub indirect_block: u32,
    pub double_indirect_block: u32,
    pub checksum: u32,
}

pub struct Directory {
    pub entries: BTreeMap<OsString, u32>,
    checksum: u32,
}

pub struct Group {
    pub block_bitmap: 
    pub data_bitmap: BitVec<Lsb0, u8>,
    pub inode_bitmap: BitVec<Lsb0, u8>,
    next_inode: Option<usize>,
    next_data_block: Option<usize>,
}
