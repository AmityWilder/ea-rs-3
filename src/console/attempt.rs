use crate::logln;
use std::fmt::Display;

#[macro_export]
macro_rules! attempt {
    ($($args:tt)+) => {
        $crate::logln!(Attempt, "{}...", format_args!($($args)+))
    };
}

pub trait Passable: Sized {
    type Pass;
    fn passing(value: Self::Pass) -> Self;
}

impl<T, E> Passable for Result<T, E> {
    type Pass = T;

    #[inline]
    fn passing(value: Self::Pass) -> Self {
        Ok(value)
    }
}

impl<T> Passable for Option<T> {
    type Pass = T;

    #[inline]
    fn passing(value: Self::Pass) -> Self {
        Some(value)
    }
}

impl Passable for () {
    type Pass = ();

    #[inline]
    fn passing((): Self::Pass) {}
}

pub trait AttemptResult: Passable {
    type Fail;

    /// If `self` is [`Ok`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    fn info(self, pass_msg: impl Display) -> Self;

    /// If `self` is [`Ok`], print `pass_msg` to the global logger as a [`super::LogType::Success`]
    fn success(self, pass_msg: impl Display) -> Self;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    fn warn(self, fail_msg: impl Display) -> Self;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`], consuming `self`
    fn issue_warning(self, fail_msg: impl Display);

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Returns `self` if [`Ok`], or `default` if [`Err`]
    fn or_warn(self, fail_msg: impl Display, default: Self::Pass) -> Self::Pass;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Returns `self` if [`Ok`], or evaluates and returns `default` if [`Err`]
    fn or_warn_with(
        self,
        fail_msg: impl Display,
        default: impl FnOnce(Self::Fail) -> Self::Pass,
    ) -> Self::Pass;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`]
    fn error(self, fail_msg: impl Display) -> Self;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`], consuming `self`
    fn issue_error(self, fail_msg: impl Display);

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as [`super::LogType::Fatal`] and panic
    fn fatal(self, fail_msg: impl Display) -> Self::Pass;
}

pub trait AttemptOption: Passable {
    /// If `self` is [`Some`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    fn info(self, pass_msg: impl Display) -> Self;

    /// If `self` is [`Some`], print `pass_msg` to the global logger as a [`super::LogType::Success`]
    fn success(self, pass_msg: impl Display) -> Self;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    fn warn(self, fail_msg: impl Display) -> Self;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`], consuming `self`
    fn issue_warning(self, fail_msg: impl Display);

    /// If `self` is [`None`] print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Returns `self` if [`Some`], or `default` if [`None`]
    fn or_warn(self, fail_msg: impl Display, default: Self::Pass) -> Self::Pass;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Returns `self` if [`Some`], or evaluates and returns `default` if [`None`]
    fn or_warn_with(
        self,
        fail_msg: impl Display,
        default: impl FnOnce() -> Self::Pass,
    ) -> Self::Pass;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`]
    fn error(self, fail_msg: impl Display) -> Self;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`], consuming `self`
    fn issue_error(self, fail_msg: impl Display);

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as [`super::LogType::Fatal`] and panic,
    fn fatal(self, fail_msg: impl Display) -> Self::Pass;
}

impl<T, E: Display> AttemptResult for Result<T, E> {
    type Fail = E;

    #[inline]
    fn info(self, pass_msg: impl Display) -> Self {
        self.inspect(|_| logln!(Info, "{pass_msg}."))
    }

    #[inline]
    fn success(self, pass_msg: impl Display) -> Self {
        self.inspect(|_| logln!(Success, "{pass_msg}."))
    }

    #[inline]
    fn warn(self, fail_msg: impl Display) -> Self {
        self.inspect_err(|e| logln!(Warning, "{fail_msg}: {e}"))
    }

    #[inline]
    fn issue_warning(self, fail_msg: impl Display) {
        if let Err(e) = self {
            logln!(Warning, "{fail_msg}: {e}");
        }
    }

    #[inline]
    fn or_warn(self, fail_msg: impl Display, default: Self::Pass) -> Self::Pass {
        self.inspect_err(|e| logln!(Warning, "{fail_msg}: {e}"))
            .unwrap_or(default)
    }

    #[inline]
    fn or_warn_with(
        self,
        fail_msg: impl Display,
        default: impl FnOnce(Self::Fail) -> Self::Pass,
    ) -> Self::Pass {
        self.inspect_err(|e| logln!(Warning, "{fail_msg}: {e}"))
            .unwrap_or_else(default)
    }

    #[inline]
    fn error(self, fail_msg: impl Display) -> Self {
        self.inspect_err(|e| logln!(Error, "{fail_msg}: {e}"))
    }

    #[inline]
    fn issue_error(self, fail_msg: impl Display) {
        if let Err(e) = self {
            logln!(Error, "{fail_msg}: {e}");
        }
    }

    #[inline]
    fn fatal(self, fail_msg: impl Display) -> Self::Pass {
        self.unwrap_or_else(|e| {
            logln!(Error, "{fail_msg}: {e}");
            panic!("fatal error: {fail_msg}: {e}")
        })
    }
}

impl<T> AttemptOption for Option<T> {
    #[inline]
    fn info(self, pass_msg: impl Display) -> Self {
        self.inspect(|_| logln!(Info, "{pass_msg}."))
    }

    #[inline]
    fn success(self, pass_msg: impl Display) -> Self {
        self.inspect(|_| logln!(Success, "{pass_msg}."))
    }

    #[inline]
    fn warn(self, fail_msg: impl Display) -> Self {
        if self.is_none() {
            logln!(Warning, "{fail_msg}.")
        }
        self
    }

    #[inline]
    fn issue_warning(self, fail_msg: impl Display) {
        if self.is_none() {
            logln!(Warning, "{fail_msg}.")
        }
    }

    #[inline]
    fn or_warn(self, fail_msg: impl Display, default: Self::Pass) -> Self::Pass {
        self.warn(fail_msg).unwrap_or(default)
    }

    #[inline]
    fn or_warn_with(
        self,
        fail_msg: impl Display,
        default: impl FnOnce() -> Self::Pass,
    ) -> Self::Pass {
        self.warn(fail_msg).unwrap_or_else(default)
    }

    #[inline]
    fn error(self, fail_msg: impl Display) -> Self {
        if self.is_none() {
            logln!(Error, "{fail_msg}.")
        }
        self
    }

    #[inline]
    fn issue_error(self, fail_msg: impl Display) {
        if self.is_none() {
            logln!(Error, "{fail_msg}.")
        }
    }

    #[inline]
    fn fatal(self, fail_msg: impl Display) -> Self::Pass {
        self.unwrap_or_else(|| {
            logln!(Error, "{fail_msg}.");
            panic!("fatal error: {fail_msg}.")
        })
    }
}
