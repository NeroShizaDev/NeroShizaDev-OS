use super::*;

#[test_case]
fn layout_plan_splits_regions_in_order() {
    let layout = NsfsLayout::plan(4096, NSFS_BLOCK_SIZE_512).unwrap();
    assert_eq!(layout.superblock_start, 0);
    assert_eq!(layout.regions[0].kind, NsfsRegionKind::Index);
    assert_eq!(layout.regions[1].kind, NsfsRegionKind::Journal);
    assert_eq!(layout.regions[2].kind, NsfsRegionKind::NhsStore);
    assert_eq!(layout.regions[3].kind, NsfsRegionKind::UserStore);
    assert_eq!(layout.regions[4].kind, NsfsRegionKind::Scratch);
    assert!(layout.regions[4].end_block() <= layout.total_blocks);
}

#[test_case]
fn normalize_path_strips_edge_slashes() {
    let mut out = [0u8; NSFS_NAME_CAPACITY];
    let len = normalize_path("/apps/demo.nhs/", &mut out).unwrap();
    assert_eq!(&out[..len], b"apps/demo.nhs");
}

#[test_case]
fn normalize_path_rejects_double_slash() {
    let mut out = [0u8; NSFS_NAME_CAPACITY];
    assert_eq!(
        normalize_path("apps//demo.nhs", &mut out),
        Err(NsfsError::InvalidPath)
    );
}

#[test_case]
fn index_insert_find_remove_roundtrip() {
    let mut index = NsfsIndex::<4>::new();
    let alloc = NsfsAllocation {
        start_block: 200,
        block_count: 8,
    };

    let slot = index
        .insert(
            "apps/demo.nhs",
            NsfsObjectKind::NhsPackage,
            0,
            alloc,
            4096,
            0xDEADBEEF,
            123,
        )
        .unwrap();
    assert_eq!(slot, 0);
    assert_eq!(index.find("/apps/demo.nhs").unwrap(), Some(0));

    index.remove("apps/demo.nhs").unwrap();
    assert_eq!(index.find("apps/demo.nhs").unwrap(), None);
}

#[test_case]
fn superblock_roundtrip_works() {
    let layout = NsfsLayout::plan(4096, NSFS_BLOCK_SIZE_512).unwrap();
    let superblock = layout.to_superblock();
    let mut bytes = [0u8; NSFS_SUPERBLOCK_SERIALIZED_LEN];
    superblock.serialize_into(&mut bytes).unwrap();
    let restored = NsfsSuperblock::deserialize_from(&bytes).unwrap();
    assert_eq!(restored, superblock);
}

#[test_case]
fn volume_persists_index_into_ramdisk() {
    let mut disk = NsfsRamDisk::<{ 512 * 1024 }>::new();
    let total_blocks = disk.total_blocks(NSFS_BLOCK_SIZE_512).unwrap();
    let mut volume = NsfsVolume::<8>::format(&mut disk, total_blocks, NSFS_BLOCK_SIZE_512).unwrap();
    volume
        .add_object(
            &mut disk,
            "kernel/banner.txt",
            NsfsObjectKind::KernelAsset,
            NsfsRegionKind::UserStore,
            0,
            b"hello kernel",
            0,
            1,
        )
        .unwrap();

    let mounted = NsfsVolume::<8>::mount(&disk).unwrap();
    assert_eq!(mounted.object_count(), 1);
    assert_eq!(
        mounted
            .index
            .get("kernel/banner.txt")
            .unwrap()
            .unwrap()
            .size_bytes,
        12
    );
}
