pub struct OptionSpan(Option<sentry::Span>);

pub trait FromTransactionOrSpan<T: Copy> {
    fn from_ts(tx: T, op: &str, description: &str) -> Self;
}

pub trait IntoOSpan<T> {
    fn into_os(self, op: &str, description: &str) -> T;
}

impl<T: Copy, U> IntoOSpan<U> for T
where
    U: FromTransactionOrSpan<T>,
    T: Copy,
{
    fn into_os(self, op: &str, description: &str) -> U {
        U::from_ts(self, op, description)
    }
}

pub trait IntoOptionSpan: IntoOSpan<OptionSpan> + Copy {}

impl<T> IntoOptionSpan for T where T: IntoOSpan<OptionSpan> + Copy {}

impl FromTransactionOrSpan<Option<&sentry::TransactionOrSpan>> for OptionSpan {
    fn from_ts(tx: Option<&sentry::TransactionOrSpan>, op: &str, description: &str) -> Self {
        Self(tx.map(|f| f.start_child(op, description)))
    }
}

impl FromTransactionOrSpan<&Option<sentry::TransactionOrSpan>> for OptionSpan {
    fn from_ts(tx: &Option<sentry::TransactionOrSpan>, op: &str, description: &str) -> Self {
        Self(tx.as_ref().map(|f| f.start_child(op, description)))
    }
}

impl FromTransactionOrSpan<&sentry::Transaction> for OptionSpan {
    fn from_ts(tx: &sentry::Transaction, op: &str, description: &str) -> Self {
        Self(Some(tx.start_child(op, description)))
    }
}

impl OptionSpan {
    pub fn set_tag<V: ToString>(&self, tag: &str, v: V) {
        self.0.as_ref().inspect(|f| f.set_tag(tag, v));
    }

    pub fn set_request(&self, request: sentry::protocol::Request) {
        self.0.as_ref().inspect(|f| f.set_request(request));
    }

    pub fn set_data(&self, key: &str, v: sentry::protocol::Value) {
        self.0.as_ref().inspect(|f| f.set_data(key, v));
    }

    pub fn set_status(&self, status: sentry::protocol::SpanStatus) {
        self.0.as_ref().inspect(|f| f.set_status(status));
    }

    pub fn finish(self) {
        self.0.and_then(|f| {
            f.finish();
            None::<sentry::Span>
        });
    }

    pub fn start_child(&self, op: &str, description: &str) -> Self {
        Self(self.0.as_ref().map(|f| f.start_child(op, description)))
    }
}
