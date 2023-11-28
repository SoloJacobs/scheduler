#![cfg(target_os = "windows")]
use windows::Win32::System::Console::GenerateConsoleCtrlEvent;
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::Console::CTRL_C_EVENT;

pub fn interrupt() {
    {
        unsafe { SetConsoleCtrlHandler(None, true).expect("ERROR") };
        unsafe { GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0).expect("ERROR") };
    }
}
