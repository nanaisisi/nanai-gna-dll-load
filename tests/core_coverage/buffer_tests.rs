use nanai_gna_dll_load::error::GnaError;
use nanai_gna_dll_load::loader::GnaLibrary;
use nanai_gna_dll_load::memory::GnaBuffer;
use nanai_gna_dll_load::types::{GNA2_STATUS_SUCCESS, Gna2MemoryTag};

use super::mock_state::*;
use super::mock_table::*;

#[test]
fn test_buffer_new_error_branches() {
    reset_mock_state();
    let lib = full_mock_library();

    // Zero size
    let err = GnaBuffer::new(&lib, 0).err().unwrap();
    assert!(matches!(err, GnaError::InvalidBufferSize(0)));
    assert!(read_mock_state(|s| s.memory_alloc_calls.is_empty()));

    // Non-success status
    with_mock_state(|s| s.memory_alloc_status = -8);
    let err = GnaBuffer::new(&lib, 256).err().unwrap();
    assert!(matches!(err, GnaError::StatusError { status: -8, .. }));

    // Null pointer returned with success status
    with_mock_state(|s| {
        s.memory_alloc_status = GNA2_STATUS_SUCCESS;
        s.memory_alloc_return_null = true;
    });
    let err = GnaBuffer::new(&lib, 256).err().unwrap();
    assert!(matches!(err, GnaError::NullPointer));
}

#[test]
fn test_buffer_new_success_slices_deref_and_drop() {
    reset_mock_state();
    let lib = full_mock_library();

    let raw_addr;
    {
        let mut buf = GnaBuffer::new(&lib, 128).unwrap();
        assert_eq!(buf.len(), 128);
        assert!(!buf.is_empty());
        raw_addr = buf.as_raw_ptr() as usize;
        assert_ne!(raw_addr, 0);

        // Mutate via as_mut_slice and read via as_slice
        buf.as_mut_slice()[0] = 0xAB;
        buf.as_mut_slice()[127] = 0xCD;
        assert_eq!(buf.as_slice()[0], 0xAB);
        assert_eq!(buf.as_slice()[127], 0xCD);

        // Mutate via DerefMut and read via Deref
        buf[1] = 0xEF;
        assert_eq!(buf[1], 0xEF);
        assert_eq!(buf.len(), 128);
        assert!(read_mock_state(|s| s.memory_free_calls.is_empty()));
    }

    assert_eq!(
        read_mock_state(|s| s.memory_free_calls.clone()),
        vec![raw_addr]
    );
}

#[test]
fn test_buffer_is_empty_when_granted_zero() {
    reset_mock_state();
    let lib = full_mock_library();

    with_mock_state(|s| s.memory_alloc_granted_override = Some(0));
    let raw_addr;
    {
        let buf = GnaBuffer::new(&lib, 64).unwrap();
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert!(buf.as_slice().is_empty());
        raw_addr = buf.as_raw_ptr() as usize;
    }
    assert_eq!(
        read_mock_state(|s| s.memory_free_calls.clone()),
        vec![raw_addr]
    );
}

#[test]
fn test_buffer_new_for_device_all_branches() {
    reset_mock_state();
    let lib = full_mock_library();

    // Zero size
    let err = GnaBuffer::new_for_device(&lib, 0, 0).err().unwrap();
    assert!(matches!(err, GnaError::InvalidBufferSize(0)));

    // Fallback to memory_alloc when memory_alloc_for_device is None
    let lib_no_dev_alloc = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    {
        let buf = GnaBuffer::new_for_device(&lib_no_dev_alloc, 3, 64).unwrap();
        assert_eq!(buf.len(), 64);
        assert_eq!(read_mock_state(|s| s.memory_alloc_calls.clone()), vec![64]);
        assert!(read_mock_state(|s| s
            .memory_alloc_for_device_calls
            .is_empty()));
    }

    // Error status from memory_alloc_for_device
    reset_mock_state();
    with_mock_state(|s| s.memory_alloc_for_device_status = -9);
    let err = GnaBuffer::new_for_device(&lib, 2, 128).err().unwrap();
    assert!(matches!(err, GnaError::StatusError { status: -9, .. }));
    assert_eq!(
        read_mock_state(|s| s.memory_alloc_for_device_calls.clone()),
        vec![(2, 128)]
    );

    // Null pointer returned with success status
    reset_mock_state();
    with_mock_state(|s| s.memory_alloc_for_device_return_null = true);
    let err = GnaBuffer::new_for_device(&lib, 1, 64).err().unwrap();
    assert!(matches!(err, GnaError::NullPointer));

    // Success
    reset_mock_state();
    let buf = GnaBuffer::new_for_device(&lib, 1, 96).unwrap();
    assert_eq!(buf.len(), 96);
    assert_eq!(
        read_mock_state(|s| s.memory_alloc_for_device_calls.clone()),
        vec![(1, 96)]
    );
}

#[test]
fn test_buffer_set_tag_all_branches() {
    reset_mock_state();

    // Missing symbol (covers library_symbol! None branch)
    let lib_no_tag = GnaLibrary::from_symbol_table(base_mock_symbol_table());
    let mut buf_no_tag = GnaBuffer::new(&lib_no_tag, 32).unwrap();
    let err = buf_no_tag.set_tag(Gna2MemoryTag::Input as u32).unwrap_err();
    assert!(matches!(err, GnaError::Other(msg) if msg.contains("memory_set_tag not supported")));
    drop(buf_no_tag);

    // Error status from memory_set_tag
    let lib = full_mock_library();
    let mut buf = GnaBuffer::new(&lib, 32).unwrap();
    let ptr_addr = buf.as_raw_ptr() as usize;

    with_mock_state(|s| s.memory_set_tag_status = -5);
    let err = buf.set_tag(Gna2MemoryTag::Output as u32).unwrap_err();
    assert!(matches!(err, GnaError::StatusError { status: -5, .. }));

    // Success status
    with_mock_state(|s| s.memory_set_tag_status = GNA2_STATUS_SUCCESS);
    buf.set_memory_tag(Gna2MemoryTag::Scratch).unwrap();
    assert_eq!(
        read_mock_state(|s| s.memory_set_tag_calls.clone()),
        vec![
            (ptr_addr, Gna2MemoryTag::Output as u32),
            (ptr_addr, Gna2MemoryTag::Scratch as u32),
        ]
    );
}

#[test]
fn test_buffer_send_across_threads() {
    reset_mock_state();
    let lib = full_mock_library();
    let mut buf = GnaBuffer::new(&lib, 32).unwrap();
    buf[0] = 0x55;

    let handle = std::thread::spawn(move || {
        assert_eq!(buf[0], 0x55);
        buf[31] = 0xAA;
        assert_eq!(buf.as_slice()[31], 0xAA);
        drop(buf);
    });
    handle.join().unwrap();
}
