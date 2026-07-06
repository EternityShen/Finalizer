use std::fs;
use std::os::unix::fs::PermissionsExt;

pub fn set_file_permissions_numeric(file_path: &str, mode: u32) -> Result<(), std::io::Error> {
    // 获取文件的元数据
    let metadata = fs::metadata(file_path)?;

    // 获取当前权限并创建一个可修改的副本
    let mut perms = metadata.permissions();

    perms.set_mode(mode);

    // 将新权限应用到文件
    fs::set_permissions(file_path, perms)?;

    Ok(())
}

pub fn inotify_init(path: &str) -> inotify::Inotify {
    let inotify = inotify::Inotify::init().unwrap();
    inotify
        .watches()
        .add(path, inotify::WatchMask::MODIFY)
        .unwrap();
    inotify
}

pub fn inotify_blockage(inotify: &mut inotify::Inotify) {
    let mut buffer = [0u8; 4096];
    loop {
        match inotify.read_events(&mut buffer) {
            Ok(events) => {
                #[allow(clippy::never_loop)]
                for _ in events {
                    break;
                }
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                panic!("读取 inotify 事件失败: {:?}", e);
            }
        }
    }
}
