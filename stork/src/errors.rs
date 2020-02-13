#[derive(Debug, Fail)]
pub enum StorkError {
    #[fail(display = "error whilst fetching link from StorkClient")]
    ClientError,
}
