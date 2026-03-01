pub enum TimeInterval {
    BySeconds,
    ByMinutes,
    ByHours,
}

#[derive(Debug)]
pub enum UrlError {
    InvalidHost,
    InvalidUrlPath,
}
