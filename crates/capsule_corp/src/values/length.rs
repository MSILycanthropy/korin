#[derive(Debug, Clone, PartialEq)]
pub enum Length {
    Cells(u16),
    Percent(f32),
    Calc(Box<CalcExpr>),
}

impl Length {
    pub const ZERO: Self = Self::Cells(0);

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    #[must_use]
    pub fn resolve(&self, parent: u16) -> u16 {
        match self {
            Self::Cells(c) => *c,
            Self::Percent(p) => (f32::from(parent) * p / 100.0).round() as u16,
            Self::Calc(expr) => expr.resolve(parent),
        }
    }
}

impl Default for Length {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Dimension {
    #[default]
    Auto,
    Length(Length),
    None,
}

impl Dimension {
    pub const ZERO: Self = Self::Length(Length::ZERO);

    #[must_use]
    pub fn resolve(&self, parent: u16) -> Option<u16> {
        match self {
            Self::Auto | Self::None => None,
            Self::Length(l) => Some(l.resolve(parent)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalcExpr {
    Cells(i16),
    Percent(f32),
    Add(Box<CalcExpr>, Box<CalcExpr>),
    Sub(Box<CalcExpr>, Box<CalcExpr>),
    Mult(Box<CalcExpr>, f32),
    Div(Box<CalcExpr>, f32),
}

impl CalcExpr {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    #[must_use]
    pub fn resolve(&self, parent: u16) -> u16 {
        self.resolve_f32(f32::from(parent)).round() as u16
    }

    fn resolve_f32(&self, parent: f32) -> f32 {
        match self {
            Self::Cells(c) => f32::from(*c),
            Self::Percent(p) => parent * p / 100.0,
            Self::Add(a, b) => a.resolve_f32(parent) + b.resolve_f32(parent),
            Self::Sub(a, b) => a.resolve_f32(parent) - b.resolve_f32(parent),
            Self::Mult(a, n) => a.resolve_f32(parent) * n,
            Self::Div(a, n) => a.resolve_f32(parent) / n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_cells() {
        let l = Length::Cells(10);
        assert_eq!(l.resolve(100), 10);
        assert_eq!(l.resolve(50), 10); // cells don't depend on parent
    }

    #[test]
    fn length_percent() {
        let l = Length::Percent(50.0);
        assert_eq!(l.resolve(100), 50);
        assert_eq!(l.resolve(80), 40);
    }

    #[test]
    fn length_percent_rounds() {
        let l = Length::Percent(50.0);
        assert_eq!(l.resolve(81), 41); // 40.5 rounds to 41
    }

    #[test]
    fn calc_add() {
        // calc(50% + 10)
        let expr = CalcExpr::Add(
            Box::new(CalcExpr::Percent(50.0)),
            Box::new(CalcExpr::Cells(10)),
        );
        let l = Length::Calc(Box::new(expr));
        assert_eq!(l.resolve(100), 60);
    }

    #[test]
    fn calc_sub() {
        // calc(100% - 10)
        let expr = CalcExpr::Sub(
            Box::new(CalcExpr::Percent(100.0)),
            Box::new(CalcExpr::Cells(10)),
        );
        let l = Length::Calc(Box::new(expr));
        assert_eq!(l.resolve(80), 70);
    }

    #[test]
    fn calc_mul_div() {
        // calc(50% * 2)
        let expr = CalcExpr::Mult(Box::new(CalcExpr::Percent(50.0)), 2.0);
        assert_eq!(expr.resolve(100), 100);

        // calc(100 / 4)
        let expr = CalcExpr::Div(Box::new(CalcExpr::Cells(100)), 4.0);
        assert_eq!(expr.resolve(0), 25);
    }

    #[test]
    fn calc_nested() {
        // calc((100% - 20) / 2)
        let inner = CalcExpr::Sub(
            Box::new(CalcExpr::Percent(100.0)),
            Box::new(CalcExpr::Cells(20)),
        );
        let expr = CalcExpr::Div(Box::new(inner), 2.0);
        let l = Length::Calc(Box::new(expr));
        assert_eq!(l.resolve(100), 40);
    }

    #[test]
    fn dimension_auto() {
        let d = Dimension::Auto;
        assert_eq!(d.resolve(100), None);
    }

    #[test]
    fn dimension_none() {
        let d = Dimension::None;
        assert_eq!(d.resolve(100), None);
    }

    #[test]
    fn dimension_length() {
        let d = Dimension::Length(Length::Cells(50));
        assert_eq!(d.resolve(100), Some(50));

        let d = Dimension::Length(Length::Percent(50.0));
        assert_eq!(d.resolve(100), Some(50));
    }
}
