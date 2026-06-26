use tokio_util::sync::CancellationToken;

pub struct CancellationGuard<'a> {
    pub(super) token: &'a CancellationToken,
}

impl<'a> CancellationGuard<'a> {
    pub fn new(token: &'a CancellationToken) -> Self {
        Self { token }
    }

    pub fn check(&self) -> bool {
        self.token.is_cancelled()
    }
}
