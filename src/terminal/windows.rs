use std::{
    fs,
    io::{self, BufWriter, Write as _},
    mem,
    os::windows::prelude::*,
    ptr,
};

use windows_sys::Win32::{
    Globalization::CP_UTF8,
    Storage::FileSystem::WriteFile,
    System::Console::{
        self, GetConsoleCP, GetConsoleMode, GetConsoleOutputCP, GetConsoleScreenBufferInfo,
        GetNumberOfConsoleInputEvents, ReadConsoleInputW, SetConsoleCP, SetConsoleMode,
        SetConsoleOutputCP, CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, INPUT_RECORD,
    },
};

use crate::{event::source::WindowsEventSource, EventSource};

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

#[derive(Debug)]
pub(crate) struct InputHandle {
    handle: OwnedHandle,
}

impl InputHandle {
    fn new<H: Into<OwnedHandle>>(h: H) -> Self {
        Self { handle: h.into() }
    }

    fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            handle: self.handle.try_clone()?,
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

    pub fn get_number_of_input_events(&mut self) -> io::Result<usize> {
        let mut num = 0;
        if unsafe { GetNumberOfConsoleInputEvents(self.as_raw_handle(), &mut num) } == 0 {
            bail!(
                "failed to read input console number of pending events: {}",
                io::Error::last_os_error()
            );
        }
        Ok(num as usize)
    }

    pub fn read_console_input(&mut self, num_events: usize) -> io::Result<Vec<INPUT_RECORD>> {
        let mut res = Vec::with_capacity(num_events);
        let zeroed: INPUT_RECORD = unsafe { mem::zeroed() };
        res.resize(num_events, zeroed);
        let mut num = 0;
        if unsafe {
            ReadConsoleInputW(
                self.as_raw_handle(),
                res.as_mut_ptr(),
                num_events as u32,
                &mut num,
            )
        } == 0
        {
            bail!(
                "failed to read console input events: {}",
                io::Error::last_os_error()
            );
        }
        unsafe { res.set_len(num as usize) };
        Ok(res)
    }
}

impl AsRawHandle for InputHandle {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle.as_raw_handle()
    }
}

#[derive(Debug)]
pub(crate) struct OutputHandle {
    handle: OwnedHandle,
}

impl OutputHandle {
    fn new<H: Into<OwnedHandle>>(h: H) -> Self {
        Self { handle: h.into() }
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

    fn get_dimensions(&mut self) -> io::Result<(u16, u16)> {
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { mem::zeroed() };
        if unsafe { GetConsoleScreenBufferInfo(self.as_raw_handle(), &mut info) } == 0 {
            bail!(
                "failed to get console screen buffer info: {}",
                io::Error::last_os_error()
            );
        }
        // NOTE: Unix dimensions are one-indexed. `+1` here to accomodate.
        let rows = info.srWindow.Bottom - info.srWindow.Top + 1;
        let cols = info.srWindow.Right - info.srWindow.Left + 1;
        Ok((rows as u16, cols as u16))
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
    // TODO: attempt to handle stdin/stdout?
    let input = InputHandle::new(
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("CONIN$")?,
    );
    let output = OutputHandle::new(
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("CONOUT$")?,
    );
    Ok((input, output))
}

#[derive(Debug)]
pub struct WindowsTerminal {
    input: InputHandle,
    output: BufWriter<OutputHandle>,
    original_input_mode: u32,
    original_output_mode: u32,
    original_input_cp: u32,
    original_output_cp: u32,
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

        Ok(Self {
            input,
            output: BufWriter::with_capacity(BUF_SIZE, output),
            original_input_mode,
            original_output_mode,
            original_input_cp,
            original_output_cp,
        })
    }
}

impl Drop for WindowsTerminal {
    fn drop(&mut self) {
        // TODO: regular VT exit stuff like the UnixTerminal: make the cursor visible, turn off
        // bracketed paste, etc...
        self.exit_alternate_screen().unwrap();
        self.output.flush().unwrap();
        self.input.set_mode(self.original_input_mode).unwrap();
        self.output
            .get_mut()
            .set_mode(self.original_output_mode)
            .unwrap();
        self.input.set_code_page(self.original_input_cp).unwrap();
        self.output
            .get_mut()
            .set_code_page(self.original_output_cp)
            .unwrap();
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
        // TODO: enabling of VT stuff: mouse, bracketed paste, etc...

        Ok(())
    }

    fn exit_raw_mode(&mut self) -> io::Result<()> {
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

    fn enter_alternate_screen(&mut self) -> io::Result<()> {
        todo!()
    }

    fn exit_alternate_screen(&mut self) -> io::Result<()> {
        todo!()
    }

    fn get_dimensions(&mut self) -> io::Result<(u16, u16)> {
        // NOTE: setting dimensions should be done by VT instead of `SetConsoleScreenBufferInfo`.
        // <https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#window-width>
        self.output.get_mut().get_dimensions()
    }

    fn event_stream(&self) -> io::Result<EventStream> {
        let source = WindowsEventSource::new(self.input.try_clone()?);
        Ok(EventStream::new(source))
    }
}
