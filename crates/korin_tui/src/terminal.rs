use std::io::{self, Stdout, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Attribute, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use korin_runtime::Runtime;
use korin_style::{Color, Modifiers};

use crate::{Buffer, Size, buffer::BufferView, renderer::render_node};

pub struct Terminal<W: Write = Stdout> {
    writer: W,
    current: Buffer,
    previous: Buffer,
}

impl Terminal<Stdout> {
    pub fn new() -> io::Result<Self> {
        let (w, h) = terminal::size()?;
        Ok(Self {
            writer: io::stdout(),
            current: Buffer::new(w, h),
            previous: Buffer::new(w, h),
        })
    }
}

impl<W> Terminal<W>
where
    W: Write,
{
    pub fn init(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(
            self.writer,
            EnterAlternateScreen,
            DisableLineWrap,
            Hide,
            Clear(ClearType::All)
        )
    }

    pub fn render(&mut self, runtime: &mut Runtime) -> io::Result<()> {
        let _span = tracing::debug_span!("render").entered();

        let size = self.size()?;

        runtime
            .render(size, |node, rect, clip| {
                let node_view = BufferView::subview(rect, clip);
                render_node(&mut self.current, &node_view, node);
            })
            .map_err(|e| io::Error::other(e.to_string()))
    }

    pub fn restore(&mut self) -> io::Result<()> {
        execute!(self.writer, Show, EnableLineWrap, LeaveAlternateScreen)?;
        terminal::disable_raw_mode()
    }

    pub fn with_writer(writer: W, width: u16, height: u16) -> Self {
        Self {
            writer,
            current: Buffer::new(width, height),
            previous: Buffer::new(width, height),
        }
    }

    pub fn size(&self) -> io::Result<Size> {
        let ct_size = terminal::size()?;

        Ok(Size::new(ct_size.0, ct_size.1))
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.current.resize(Size::new(width, height));
        self.previous.resize(Size::new(width, height));
    }

    #[must_use]
    pub const fn buffer(&mut self) -> &mut Buffer {
        &mut self.current
    }

    pub fn flush(&mut self) -> io::Result<()> {
        queue!(
            self.writer,
            SetForegroundColor(Color::Reset.into()),
            SetBackgroundColor(Color::Reset.into())
        )?;

        let mut last_foreground = Color::Reset;
        let mut last_background = Color::Reset;
        let mut last_modifiers = Modifiers::NONE;

        for (x, y, cell) in self.current.diff(&self.previous) {
            queue!(self.writer, MoveTo(x, y))?;

            if cell.foreground != last_foreground {
                queue!(self.writer, SetForegroundColor(cell.foreground.into()))?;

                last_foreground = cell.foreground;
            }

            if cell.background != last_background {
                queue!(self.writer, SetBackgroundColor(cell.background.into()))?;

                last_background = cell.background;
            }

            if cell.modifiers != last_modifiers {
                queue_modifiers(&mut self.writer, last_modifiers, cell.modifiers)?;

                last_modifiers = cell.modifiers;
            }

            queue!(self.writer, Print(cell.symbol))?;
        }

        self.writer.flush()?;
        std::mem::swap(&mut self.current, &mut self.previous);
        self.current.clear();

        Ok(())
    }
}

impl<W> Drop for Terminal<W>
where
    W: Write,
{
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn queue_modifiers(w: &mut impl Write, old: Modifiers, new: Modifiers) -> io::Result<()> {
    if old.contains(Modifiers::BOLD) && !new.contains(Modifiers::BOLD) {
        queue!(w, SetAttribute(Attribute::NormalIntensity))?;
    }
    if old.contains(Modifiers::DIM) && !new.contains(Modifiers::DIM) {
        queue!(w, SetAttribute(Attribute::NormalIntensity))?;
    }
    if old.contains(Modifiers::ITALIC) && !new.contains(Modifiers::ITALIC) {
        queue!(w, SetAttribute(Attribute::NoItalic))?;
    }
    if old.contains(Modifiers::UNDERLINE) && !new.contains(Modifiers::UNDERLINE) {
        queue!(w, SetAttribute(Attribute::NoUnderline))?;
    }

    if new.contains(Modifiers::BOLD) && !old.contains(Modifiers::BOLD) {
        queue!(w, SetAttribute(Attribute::Bold))?;
    }
    if new.contains(Modifiers::DIM) && !old.contains(Modifiers::DIM) {
        queue!(w, SetAttribute(Attribute::Dim))?;
    }
    if new.contains(Modifiers::ITALIC) && !old.contains(Modifiers::ITALIC) {
        queue!(w, SetAttribute(Attribute::Italic))?;
    }
    if new.contains(Modifiers::UNDERLINE) && !old.contains(Modifiers::UNDERLINE) {
        queue!(w, SetAttribute(Attribute::Underlined))?;
    }

    Ok(())
}
