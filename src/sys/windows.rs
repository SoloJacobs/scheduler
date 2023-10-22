#![cfg(target_os = "windows")]

pub fn interrupt(pid: str) {
    use windows::Win32::System::Console;
    {
        unsafe { Console::AttachConsole(pid) }.expect("PLEASE");
        unsafe { Console::GenerateConsoleCtrlEvent(Console::CTRL_C_EVENT, pid) }
            .expect("FAILED");
    }
}
