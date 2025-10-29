// use bitvec::vec::BitVec;

pub const DIRECT_POINTERS: u64 = 12;

// pub struct SuperBlock {
//     pub magic: u32,
//     pub block_size: u32,
//     pub created_at: u64,
//     pub modified_at: Option<u64>,
//     pub last_mounted_at: Option<u64>,
//     pub block_count: u32,
//     pub inode_count: u32,
//     pub free_blocks: u32,
//     pub free_inodes: u32,
//     pub groups: u32,
//     pub blocks_per_group: u32,
//     pub inodes_per_group: u32,
//     pub uid: u32,
//     pub gid: u32,
//     pub checksum: u32,
// }
#[repr(C)]
pub struct SuperBlock {
    pub inodes_count: u32,
    pub blocks_count: u32,
    pub r_blocks_count: u32,
    pub free_blocks_count: u32,
    pub free_inodes_count: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,
    pub log_frag_size: u32,
    pub blocks_per_group: u32,
    pub frags_per_group: u32,
    pub inodes_per_group: u32,
    pub mtime: u32,
    pub wtime: u32,
    pub mnt_count: u16,
    pub max_mnt_count: u16,
    pub magic: u16,
    pub state: u16,
    pub errors: u16,
    pub minor_rev_level: u16,
    pub lastcheck: u32,
    pub checkinterval: u32,
    pub creator_os: u32,
    pub rev_level: u32,
    pub def_resuid: u16,
    pub def_resgid: u16,
}

const SUPERBLOCK_SIZE: usize = core::mem::size_of::<SuperBlock>();

pub struct SuperblockStorage {
    data: [u8; SUPERBLOCK_SIZE],
}

impl SuperblockStorage {
    pub fn new() -> Self {
        SuperblockStorage {
            data: [0; SUPERBLOCK_SIZE],
        }
    }

    pub fn initialize(&mut self) {
        let superblock = SuperBlock {
            inodes_count: 1024,
            blocks_count: 2048,
            r_blocks_count: 0,
            free_blocks_count: 2047,
            free_inodes_count: 1023,
            first_data_block: 0,
            log_block_size: 0, // 1024 bytes
            log_frag_size: 0,
            blocks_per_group: 2048,
            frags_per_group: 0,
            inodes_per_group: 128,
            mtime: 0,
            wtime: 0,
            mnt_count: 0,
            max_mnt_count: 0,
            magic: 0xEF53, // ext2 magic number
            state: 1,
            errors: 0,
            minor_rev_level: 0,
            lastcheck: 0,
            checkinterval: 0,
            creator_os: 0,
            rev_level: 0,
            def_resuid: 0,
            def_resgid: 0,
        };

        let bytes: &[u8] = unsafe {
            core::slice::from_raw_parts(&superblock as *const _ as *const u8, SUPERBLOCK_SIZE)
        };

        self.data.copy_from_slice(bytes);
    }

    pub fn get_superblock(&self) -> &SuperBlock {
        unsafe { &*(self.data.as_ptr() as *const SuperBlock) }
    }
}

// table
pub struct INode {
    pub mode: u32, // differentiates with files, directories, symbolic links
    pub hard_links: u16,
    pub user_id: u32,
    pub group_id: u32,
    pub block_count: u32, // should be in 512 bytes blocks
    pub size: u64,
    pub created_at: u64,
    pub accessed_at: Option<i64>,
    pub modified_at: Option<i64>,
    pub changed_at: Option<i64>,
    // i_block
    pub direct_blocks: [u32; DIRECT_POINTERS as usize],
    pub indirect_block: u32,
    pub double_indirect_block: u32,
    pub checksum: u32,
}

// stored as a data block that gets referenced as
// pub struct Directory {
//     pub entries: BTreeMap<OsString, u32>, // linked list entries of file entries
//     checksum: u32,
// }

// pub struct GroupDescriptorTable {
//     pub block_bitmap: BitVec<Lsb0, u8>, // actual data
//     pub inode_bitmap: BitVec<Lsb0, u8>, // contains all inodes
//     pub inode_table: u32,
//     pub free_blocks_count: u16,
//     pub free_inodes_count: u16,
//     pub used_dirs_count: u16,
//     pub pad: u16,
//     pub reserved: [u8; 12],
//     next_inode: Option<usize>,
//     next_data_block: Option<usize>,
// }
