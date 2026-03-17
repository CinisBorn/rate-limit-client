pub enum TimeInterval {
    BySeconds,
    ByMinutes,
    ByHours,
}

impl Copy for TimeInterval {}

impl Clone for TimeInterval {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Debug)]
pub enum UrlError {
    InvalidHost,
    InvalidUrlPath,
}
