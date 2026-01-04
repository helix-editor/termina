use std::{
    fmt,
    fs::{self, File},
    io::{self, BufWriter, IsTerminal as _, Write as _},
    mem,
    os::windows::prelude::*,
    ptr,
};

use windows_sys::Win32::{
    Storage::FileSystem::WriteFile,
    System::Console::{
        self, GetConsoleCP, GetConsoleMode, GetConsoleOutputCP, GetConsoleScreenBufferInfo,
        GetNumberOfConsoleInputEvents, ReadConsoleInputA, SetConsoleCP, SetConsoleMode,
        SetConsoleOutputCP, CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, INPUT_RECORD,
    },
};

use crate::{event::source::WindowsEventSource, Event, EventReader, OneBased, WindowSize};

use super::Terminal;

macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err(::std::io::Error::new(::std::io::ErrorKind::Other, $msg))
    };
    ($fmt:expr $(,)?, $($arg:tt)*) => {
        return Err(::std::io::Error::new(::std::io::ErrorKind::Other, format!($fmt, $($arg)*)))
    };
}

const BUF_SIZE: usize = 128;

type CodePageID = u32;
/// The code page ID for UTF-8 encoding.
/// This is the same as `windows_sys::Win32::Globalization::CP_UTF8`. It is copied here rather
/// than `use`d because it is the only thing we want from the globalization API. Avoiding the
/// `Win32_Globalization` feature for `windows_sys` saves a fair amount of compilation time.
/// And it's unimaginable that Windows would ever change a constant like this given their passion
/// for backwards compatibility.
const CP_UTF8: CodePageID = 65001;

// CREDIT: Like the Unix terminal module this is mainly based on WezTerm code (except for the
// event source parts in `src/event/source/windows.rs` which reaches into these functions).
// <https://github.com/wezterm/wezterm/blob/a87358516004a652ad840bc1661bdf65ffc89b43/filedescriptor/src/windows.rs>
// This crate however uses `windows-sys` instead of `winapi` and has a slightly different API for
// the `InputHandle` and `OutputHandle`.

#[derive(Debug)]
pub enum Handle {
    Owned(OwnedHandle),
    Borrowed(BorrowedHandle<'static>),
}

impl AsRawHandle for Handle {
    fn as_raw_handle(&self) -> RawHandle {
        match self {
            Self::Owned(handle) => handle.as_raw_handle(),
            Self::Borrowed(handle) => handle.as_raw_handle(),
        }
    }
}

impl Handle {
    pub fn stdin() -> Self {
        let stdin = io::stdin().as_raw_handle();
        Self::Borrowed(unsafe { BorrowedHandle::borrow_raw(stdin) })
    }

    pub fn stdout() -> Self {
        let stdout = io::stdout().as_raw_handle();
        Self::Borrowed(unsafe { BorrowedHandle::borrow_raw(stdout) })
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        let this = match self {
            Self::Owned(handle) => Self::Owned(handle.try_clone()?),
            Self::Borrowed(handle) => Self::Borrowed(*handle),
        };
        Ok(this)
    }
}

impl From<File> for Handle {
    fn from(file: File) -> Self {
        Self::Owned(OwnedHandle::from(file))
    }
}

pub(crate) struct InputHandle {
    handle: Handle,
    input_buf: Vec<INPUT_RECORD>,
}

impl fmt::Debug for InputHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputHandle")
            .field("handle", &self.handle)
            .finish_non_exhaustive()
    }
}

impl InputHandle {
    fn new(handle: Handle) -> Self {
        let mut input_buf = Vec::with_capacity(BUF_SIZE);
        let zeroed: INPUT_RECORD = unsafe { mem::zeroed() };
        input_buf.resize(BUF_SIZE, zeroed);
        Self { handle, input_buf }
    }

    fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            handle: self.handle.try_clone()?,
            input_buf: self.input_buf.clone(),
        })
    }

    fn get_mode(&self) -> io::Result<CONSOLE_MODE> {
        let mut mode = 0;
        if unsafe { GetConsoleMode(self.as_raw_handle(), &mut mode) } == 0 {
            bail!(
                "failed to get input console mode: {}",
                io::Error::last_os_error()
            );
        }
        Ok(mode)
    }

    fn set_mode(&mut self, mode: CONSOLE_MODE) -> io::Result<()> {
        if unsafe { SetConsoleMode(self.as_raw_handle(), mode) } == 0 {
            bail!(
                "failed to set input console mode: {}",
                io::Error::last_os_error()
            );
        }

        Ok(())
    }

    fn get_code_page(&self) -> io::Result<CodePageID> {
        let cp = unsafe { GetConsoleCP() };
        if cp == 0 {
            bail!(
                "failed to get input console codepage ID: {}",
                io::Error::last_os_error()
            );
        }
        Ok(cp)
    }

    fn set_code_page(&mut self, cp: CodePageID) -> io::Result<()> {
        if unsafe { SetConsoleCP(cp) } == 0 {
            bail!(
                "failed to set input console codepage ID: {}",
                io::Error::last_os_error()
            );
        }
        Ok(())
    }

    pub fn has_pending_input_events(&mut self) -> io::Result<bool> {
        let mut num = 0;
        // Since we use UTF-8 code pages and call ReadConsoleInputA to read UTF-8 data,
        // we can't rely on the result from GetNumberOfConsoleInputEvents.
        // Its return value matches the result from ReadConsoleInputW, which may not be the
        // same when typing some Unicode values.
        // Instead, we can just use it as a quick check to see if events are available.
        if unsafe { GetNumberOfConsoleInputEvents(self.as_raw_handle(), &mut num) } == 0 {
            bail!(
                "failed to read input console number of pending events: {}",
                io::Error::last_os_error()
            );
        }
        Ok(num > 0)
    }

    pub fn read_console_input(&mut self) -> io::Result<&[INPUT_RECORD]> {
        let mut num = 0;
        // NOTE: <https://learn.microsoft.com/en-us/windows/console/classic-vs-vt#unicode>
        // > UTF-8 support in the console can be utilized via the A variant of Console APIs
        // > against console handles after setting the codepage to 65001 or CP_UTF8 with the
        // > SetConsoleOutputCP and SetConsoleCP methods, as appropriate.
        if unsafe {
            ReadConsoleInputA(
                self.as_raw_handle(),
                self.input_buf.as_mut_ptr(),
                self.input_buf.capacity() as u32,
                &mut num,
            )
        } == 0
        {
            bail!(
                "failed to read console input events: {}",
                io::Error::last_os_error()
            );
        }
        unsafe { self.input_buf.set_len(num as usize) };
        Ok(&self.input_buf)
    }
}

impl AsRawHandle for InputHandle {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle.as_raw_handle()
    }
}

#[derive(Debug)]
pub struct OutputHandle {
    handle: Handle,
}

impl OutputHandle {
    fn new(handle: Handle) -> Self {
        Self { handle }
    }

    fn get_mode(&self) -> io::Result<CONSOLE_MODE> {
        let mut mode = 0;
        if unsafe { GetConsoleMode(self.as_raw_handle(), &mut mode) } == 0 {
            bail!(
                "failed to get output console mode: {}",
                io::Error::last_os_error()
            );
        }
        Ok(mode)
    }

    fn set_mode(&mut self, mode: CONSOLE_MODE) -> io::Result<()> {
        if unsafe { SetConsoleMode(self.as_raw_handle(), mode) } == 0 {
            bail!(
                "failed to set output console mode: {}",
                io::Error::last_os_error()
            );
        }

        Ok(())
    }

    fn get_code_page(&self) -> io::Result<CodePageID> {
        let cp = unsafe { GetConsoleOutputCP() };
        if cp == 0 {
            bail!(
                "failed to get output console codepage ID: {}",
                io::Error::last_os_error()
            );
        }
        Ok(cp)
    }

    fn set_code_page(&mut self, cp: CodePageID) -> io::Result<()> {
        if unsafe { SetConsoleOutputCP(cp) } == 0 {
            bail!(
                "failed to set output console codepage ID: {}",
                io::Error::last_os_error()
            );
        }
        Ok(())
    }

    fn get_dimensions(&self) -> io::Result<WindowSize> {
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { mem::zeroed() };
        if unsafe { GetConsoleScreenBufferInfo(self.as_raw_handle(), &mut info) } == 0 {
            bail!(
                "failed to get console screen buffer info: {}",
                io::Error::last_os_error()
            );
        }
        let rows = OneBased::from_zero_based((info.srWindow.Bottom - info.srWindow.Top) as u16);
        let cols = OneBased::from_zero_based((info.srWindow.Right - info.srWindow.Left) as u16);
        Ok(WindowSize {
            rows: rows.get(),
            cols: cols.get(),
            pixel_width: None,
            pixel_height: None,
        })
    }
}

impl AsRawHandle for OutputHandle {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle.as_raw_handle()
    }
}

impl io::Write for OutputHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut num_written = 0;
        if unsafe {
            WriteFile(
                self.as_raw_handle(),
                buf.as_ptr(),
                buf.len() as u32,
                &mut num_written,
                ptr::null_mut(),
            )
        } == 0
        {
            Err(io::Error::last_os_error())
        } else {
            Ok(num_written as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn open_pty() -> io::Result<(InputHandle, OutputHandle)> {
    let input = if io::stdin().is_terminal() {
        Handle::stdin()
    } else {
        open_file("CONIN$")?.into()
    };
    let output = if io::stdout().is_terminal() {
        Handle::stdout()
    } else {
        open_file("CONOUT$")?.into()
    };
    Ok((InputHandle::new(input), OutputHandle::new(output)))
}

fn open_file(path: &str) -> io::Result<File> {
    fs::OpenOptions::new().read(true).write(true).open(path)
}

// CREDIT: Again, like the UnixTerminal in the unix module this is mostly based on WezTerm but
// only covers the parts not related to the event source.
// <https://github.com/wezterm/wezterm/blob/a87358516004a652ad840bc1661bdf65ffc89b43/termwiz/src/terminal/windows.rs#L482-L860>
// Also, the legacy Console API is not implemented.

#[derive(Debug)]
pub struct WindowsTerminal {
    input: InputHandle,
    output: BufWriter<OutputHandle>,
    reader: EventReader,
    original_input_mode: CONSOLE_MODE,
    original_output_mode: CONSOLE_MODE,
    original_input_cp: CodePageID,
    original_output_cp: CodePageID,
    has_panic_hook: bool,
}

impl WindowsTerminal {
    pub fn new() -> io::Result<Self> {
        let (mut input, mut output) = open_pty()?;

        let original_input_mode = input.get_mode()?;
        let original_output_mode = output.get_mode()?;
        let original_input_cp = input.get_code_page()?;
        let original_output_cp = output.get_code_page()?;
        input.set_code_page(CP_UTF8)?;
        output.set_code_page(CP_UTF8)?;

        // Enable VT processing for the output handle.
        let desired_output_mode = original_output_mode
            | Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING
            | Console::DISABLE_NEWLINE_AUTO_RETURN;
        if output.set_mode(desired_output_mode).is_err() {
            bail!("virtual terminal processing could not be enabled for the output handle");
        }
        // And now the input handle too.
        let desired_input_mode = original_input_mode | Console::ENABLE_VIRTUAL_TERMINAL_INPUT;
        if input.set_mode(desired_input_mode).is_err() {
            bail!("virtual terminal processing could not be enabled for the input handle");
        }

        let reader = EventReader::new(WindowsEventSource::new(input.try_clone()?)?);

        Ok(Self {
            input,
            output: BufWriter::with_capacity(BUF_SIZE, output),
            reader,
            original_input_mode,
            original_output_mode,
            original_input_cp,
            original_output_cp,
            has_panic_hook: false,
        })
    }
}

impl Terminal for WindowsTerminal {
    fn enter_raw_mode(&mut self) -> io::Result<()> {
        let mode = self.output.get_mut().get_mode()?;
        self.output
            .get_mut()
            .set_mode(mode | Console::DISABLE_NEWLINE_AUTO_RETURN)
            .ok();
        let mode = self.input.get_mode()?;
        self.input.set_mode(
            (mode
                & !(Console::ENABLE_ECHO_INPUT
                    | Console::ENABLE_LINE_INPUT
                    | Console::ENABLE_PROCESSED_INPUT))
                | Console::ENABLE_MOUSE_INPUT
                | Console::ENABLE_WINDOW_INPUT,
        )?;

        Ok(())
    }

    fn enter_cooked_mode(&mut self) -> io::Result<()> {
        let mode = self.output.get_mut().get_mode()?;
        self.output
            .get_mut()
            .set_mode(mode & !Console::DISABLE_NEWLINE_AUTO_RETURN)
            .ok();

        let mode = self.input.get_mode()?;
        self.input.set_mode(
            (mode & !(Console::ENABLE_MOUSE_INPUT | Console::ENABLE_WINDOW_INPUT))
                | Console::ENABLE_ECHO_INPUT
                | Console::ENABLE_LINE_INPUT
                | Console::ENABLE_PROCESSED_INPUT,
        )?;
        Ok(())
    }

    fn get_dimensions(&self) -> io::Result<WindowSize> {
        // NOTE: setting dimensions should be done by VT instead of `SetConsoleScreenBufferInfo`.
        // <https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#window-width>
        self.output.get_ref().get_dimensions()
    }

    fn event_reader(&self) -> EventReader {
        self.reader.clone()
    }

    fn poll<F: Fn(&Event) -> bool>(
        &self,
        filter: F,
        timeout: Option<std::time::Duration>,
    ) -> io::Result<bool> {
        self.reader.poll(timeout, filter)
    }

    fn read<F: Fn(&Event) -> bool>(&self, filter: F) -> io::Result<Event> {
        self.reader.read(filter)
    }

    fn set_panic_hook(&mut self, f: impl Fn(&mut OutputHandle) + Send + Sync + 'static) {
        let original_input_cp = self.original_input_cp;
        let original_input_mode = self.original_input_mode;
        let original_output_cp = self.original_output_cp;
        let original_output_mode = self.original_output_mode;
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if let Ok((mut input, mut output)) = open_pty() {
                f(&mut output);
                let _ = input.set_code_page(original_input_cp);
                let _ = input.set_mode(original_input_mode);
                let _ = output.set_code_page(original_output_cp);
                let _ = output.set_mode(original_output_mode);
            }
            hook(info);
        }));
        self.has_panic_hook = true;
    }
}

impl Drop for WindowsTerminal {
    fn drop(&mut self) {
        if !self.has_panic_hook || !std::thread::panicking() {
            let _ = self.flush();
            let _ = self.input.set_code_page(self.original_input_cp);
            let _ = self.output.get_mut().set_code_page(self.original_output_cp);
            let _ = self.input.set_mode(self.original_input_mode);
            let _ = self.output.get_mut().set_mode(self.original_output_mode);
        }
    }
}

impl io::Write for WindowsTerminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}
