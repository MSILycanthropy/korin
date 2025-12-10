pub struct BorderSymbols {
    pub h: char,
    pub v: char,
    pub tl: char,
    pub tr: char,
    pub bl: char,
    pub br: char,
}

impl BorderSymbols {
    pub const PLAIN: Self = Self {
        h: '─',
        v: '│',
        tl: '┌',
        tr: '┐',
        bl: '└',
        br: '┘',
    };

    pub const ROUNDED: Self = Self {
        h: '─',
        v: '│',
        tl: '╭',
        tr: '╮',
        bl: '╰',
        br: '╯',
    };

    pub const DOUBLE: Self = Self {
        h: '═',
        v: '║',
        tl: '╔',
        tr: '╗',
        bl: '╚',
        br: '╝',
    };

    pub const THICK: Self = Self {
        h: '━',
        v: '┃',
        tl: '┏',
        tr: '┓',
        bl: '┗',
        br: '┛',
    };
}
