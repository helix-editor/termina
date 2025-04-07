use std::fmt::{self, Display};

use crate::style::CursorStyle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dcs {
    // DECRQSS: <https://vt100.net/docs/vt510-rm/DECRQSS.html>
    Request(DcsRequest),
    // DECRPSS: <https://vt100.net/docs/vt510-rm/DECRPSS.html>
    Response {
        is_request_valid: bool,
        value: DcsResponse,
    },
}

impl Display for Dcs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // DCS
        f.write_str(super::DCS)?;
        match self {
            // DCS $ q D...D ST
            Self::Request(request) => write!(f, "$q{request}")?,
            // DCS Ps $ r D...D ST
            Self::Response {
                is_request_valid,
                value,
            } => write!(f, "{}$r{value}", if *is_request_valid { 1 } else { 0 })?,
        }
        // ST
        f.write_str(super::ST)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DcsRequest {
    ActiveStatusDisplay,
    AttributeChangeExtent,
    CharacterAttribute,
    ConformanceLevel,
    ColumnsPerPage,
    LinesPerPage,
    NumberOfLinesPerScreen,
    StatusLineType,
    LeftAndRightMargins,
    TopAndBottomMargins,
    /// SGR
    GraphicRendition,
    SetUpLanguage,
    PrinterType,
    RefreshRate,
    DigitalPrintedDataType,
    ProPrinterCharacterSet,
    CommunicationSpeed,
    CommunicationPort,
    ScrollSpeed,
    CursorStyle,
    KeyClickVolume,
    WarningBellVolume,
    MarginBellVolume,
    LockKeyStyle,
    FlowControlType,
    DisconnectDelayTime,
    TransmitRateLimit,
    PortParameter,
}

impl Display for DcsRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActiveStatusDisplay => f.write_str("$}"),
            Self::AttributeChangeExtent => write!(f, "*x"),
            Self::CharacterAttribute => write!(f, "\"q"),
            Self::ConformanceLevel => write!(f, "\"p"),
            Self::ColumnsPerPage => write!(f, "$|"),
            Self::LinesPerPage => write!(f, "t"),
            Self::NumberOfLinesPerScreen => write!(f, "*|"),
            Self::StatusLineType => write!(f, "$~"),
            Self::LeftAndRightMargins => write!(f, "s"),
            Self::TopAndBottomMargins => write!(f, "r"),
            Self::GraphicRendition => write!(f, "m"),
            Self::SetUpLanguage => write!(f, "p"),
            Self::PrinterType => write!(f, "$s"),
            Self::RefreshRate => write!(f, "\"t"),
            Self::DigitalPrintedDataType => write!(f, "(p"),
            Self::ProPrinterCharacterSet => write!(f, "*p"),
            Self::CommunicationSpeed => write!(f, "*r"),
            Self::CommunicationPort => write!(f, "*u"),
            // NOTE: space char is intentional - written as SP in
            // <https://vt100.net/docs/vt510-rm/DECRPSS.html>
            Self::ScrollSpeed => write!(f, " p"),
            Self::CursorStyle => write!(f, " q"),
            Self::KeyClickVolume => write!(f, " r"),
            Self::WarningBellVolume => write!(f, " t"),
            Self::MarginBellVolume => write!(f, " u"),
            Self::LockKeyStyle => write!(f, " v"),
            Self::FlowControlType => write!(f, "*s"),
            Self::DisconnectDelayTime => write!(f, "$q"),
            Self::TransmitRateLimit => write!(f, "\"u"),
            Self::PortParameter => write!(f, "+w"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DcsResponse {
    /// SGR
    GraphicRendition(Vec<super::csi::Sgr>),
    CursorStyle(CursorStyle),
    // There are others but adding them would mean adding a lot of parsing code...
}

impl Display for DcsResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GraphicRendition(sgrs) => {
                let mut first = true;
                for sgr in sgrs {
                    if !first {
                        write!(f, ";")?;
                    }
                    first = false;
                    write!(f, "{sgr}")?;
                }
                Ok(())
            }
            Self::CursorStyle(style) => write!(f, "{style} q"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encoding() {
        assert_eq!(
            Dcs::Request(DcsRequest::GraphicRendition).to_string(),
            "\x1bP$qm\x1b\\"
        );
        assert_eq!(
            Dcs::Request(DcsRequest::CursorStyle).to_string(),
            "\x1bP$q q\x1b\\"
        );
    }
}
