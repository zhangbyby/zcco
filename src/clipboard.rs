#[cfg(not(target_os = "macos"))]
compile_error!("zcco supports macOS only");

use objc2::rc::{Retained, autoreleasepool};
use objc2::{ClassType, msg_send};
use objc2_app_kit::NSPasteboard;
use objc2_foundation::NSString;

pub fn copy(text: &str) -> Result<(), String> {
    autoreleasepool(|_| {
        // SAFETY: this is NSPasteboard's documented class method and return
        // type. Accept nil so an unavailable pasteboard service is an error,
        // rather than the panic in the generated non-null binding.
        let board: Option<Retained<NSPasteboard>> =
            unsafe { msg_send![NSPasteboard::class(), generalPasteboard] };
        let board = board.ok_or("无法连接 macOS 系统剪贴板服务，请在桌面会话中运行")?;
        copy_to(&board, text)
    })
}

fn copy_to(board: &NSPasteboard, text: &str) -> Result<(), String> {
    // Own the new pasteboard contents before setting UTF-8 text. The system
    // pasteboard server retains the data after this command exits.
    board.clearContents();
    if board.setString_forType(
        &NSString::from_str(text),
        &NSString::from_str("public.utf8-plain-text"),
    ) {
        Ok(())
    } else {
        Err("写入 macOS 系统剪贴板失败".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::NSString;

    #[test]
    fn native_pasteboard_round_trips_text_without_external_commands() {
        autoreleasepool(|_| {
            let board = NSPasteboard::pasteboardWithUniqueName();
            let name = board.name();
            for text in ["  你好\nworld  ", "", &"大段文字\n".repeat(100_000)] {
                copy_to(&board, text).unwrap();
                let reopened = NSPasteboard::pasteboardWithName(&name);
                let actual = reopened
                    .stringForType(&NSString::from_str("public.utf8-plain-text"))
                    .unwrap();
                assert_eq!(actual.to_string(), text);
            }
            board.clearContents();
        });
    }
}
