#![allow(dead_code, reason = "library-like")]
use crate::logln;
use std::fmt::Display;

#[macro_export]
macro_rules! attempt {
    ($($args:tt)+) => {
        $crate::logln!(Attempt, "{}...", format_args!($($args)+))
    };
}

pub trait Passable {
    type Pass;
    fn passing(value: Self::Pass) -> Self;
}

pub trait Failable {
    type Fail;
    fn failing(value: Self::Fail) -> Self;
}

impl<T, E> Passable for Result<T, E> {
    type Pass = T;

    #[inline]
    fn passing(value: Self::Pass) -> Self {
        Ok(value)
    }
}

impl<T, E> Failable for Result<T, E> {
    type Fail = E;

    #[inline]
    fn failing(value: Self::Fail) -> Self {
        Err(value)
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

pub trait Loggable<T: ?Sized = ()> {
    type Proxy: Display;

    fn display(self, ctx: &T) -> Self::Proxy;
}

impl<T: ?Sized> Loggable<T> for &str {
    type Proxy = Self;

    #[inline]
    fn display(self, _: &T) -> Self::Proxy {
        self
    }
}

impl<T: ?Sized> Loggable<T> for String {
    type Proxy = Self;

    #[inline]
    fn display(self, _: &T) -> Self::Proxy {
        self
    }
}

impl<T: ?Sized> Loggable<T> for std::fmt::Arguments<'_> {
    type Proxy = Self;

    #[inline]
    fn display(self, _: &T) -> Self::Proxy {
        self
    }
}

impl<T: ?Sized, U: Display, F: FnOnce(&T) -> U> Loggable<T> for F {
    type Proxy = U;

    #[inline]
    fn display(self, ctx: &T) -> Self::Proxy {
        self(ctx)
    }
}

pub trait AttemptResult: Passable + Failable {
    /// If `self` is [`Ok`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    fn ok_info(self, pass_msg: impl Loggable<Self::Pass>) -> Self;

    /// If `self` is [`Err`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    fn err_info(self, fail_msg: impl Loggable<Self::Fail>) -> Self;

    /// Print `msg` to the global logger as [`super::LogType::Info`], regardless of `self`
    fn info(self, msg: impl Loggable<Self>) -> Self;

    /// If `self` is [`Ok`], print `pass_msg` to the global logger as a [`super::LogType::Success`]
    fn success(self, pass_msg: impl Loggable<Self::Pass>) -> Self;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    fn warn(self, fail_msg: impl Loggable<Self::Fail>) -> Self;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`], consuming `self`
    fn issue_warning(self, fail_msg: impl Loggable<Self::Fail>);

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Returns `self` if [`Ok`], or `default` if [`Err`]
    fn or_warn(self, fail_msg: impl Loggable<Self::Fail>, default: Self::Pass) -> Self::Pass;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Returns `self` if [`Ok`], or evaluates and returns `default` if [`Err`]
    fn or_warn_with(
        self,
        fail_msg: impl Loggable<Self::Fail>,
        default: impl FnOnce(Self::Fail) -> Self::Pass,
    ) -> Self::Pass;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`]
    fn error(self, fail_msg: impl Loggable<Self::Fail>) -> Self;

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`], consuming `self`
    fn issue_error(self, fail_msg: impl Loggable<Self::Fail>);

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as [`super::LogType::Fatal`] and panic
    fn fatal(self, fail_msg: impl Loggable<Self::Fail>) -> Self::Pass;
}

pub trait AttemptOption: Passable {
    /// If `self` is [`Some`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    fn some_info(self, pass_msg: impl Loggable<Self::Pass>) -> Self;

    /// If `self` is [`None`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    fn none_info(self, pass_msg: impl Loggable<()>) -> Self;

    /// Print `pass_msg` to the global logger as [`super::LogType::Info`], regardless of `self`
    fn info(self, msg: impl Loggable<Self>) -> Self;

    /// If `self` is [`Some`], print `pass_msg` to the global logger as a [`super::LogType::Success`]
    fn success(self, pass_msg: impl Loggable<Self::Pass>) -> Self;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    fn warn(self, fail_msg: impl Loggable<()>) -> Self;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`], consuming `self`
    fn issue_warning(self, fail_msg: impl Loggable<()>);

    /// If `self` is [`None`] print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Returns `self` if [`Some`], or `default` if [`None`]
    fn or_warn(self, fail_msg: impl Loggable<()>, default: Self::Pass) -> Self::Pass;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Returns `self` if [`Some`], or evaluates and returns `default` if [`None`]
    fn or_warn_with(
        self,
        fail_msg: impl Loggable<()>,
        default: impl FnOnce() -> Self::Pass,
    ) -> Self::Pass;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`]
    fn error(self, fail_msg: impl Loggable<()>) -> Self;

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`], consuming `self`
    fn issue_error(self, fail_msg: impl Loggable<()>);

    /// If `self` is [`None`], print `fail_msg` and the error to the global logger
    /// as [`super::LogType::Fatal`] and panic,
    fn fatal(self, fail_msg: impl Loggable<()>) -> Self::Pass;
}

impl<T, E: Display> AttemptResult for Result<T, E> {
    #[inline]
    fn ok_info(self, pass_msg: impl Loggable<Self::Pass>) -> Self {
        self.inspect(|v| logln!(Info, "{}.", pass_msg.display(v)))
    }

    #[inline]
    fn err_info(self, fail_msg: impl Loggable<Self::Fail>) -> Self {
        self.inspect_err(|e| logln!(Info, "{}.", fail_msg.display(e)))
    }

    #[inline]
    fn info(self, msg: impl Loggable<Self>) -> Self {
        logln!(Info, "{}.", msg.display(&self));
        self
    }

    #[inline]
    fn success(self, pass_msg: impl Loggable<Self::Pass>) -> Self {
        self.inspect(|v| logln!(Success, "{}.", pass_msg.display(v)))
    }

    #[inline]
    fn warn(self, fail_msg: impl Loggable<Self::Fail>) -> Self {
        self.inspect_err(|e| logln!(Warning, "{}: {e}", fail_msg.display(e)))
    }

    #[inline]
    fn issue_warning(self, fail_msg: impl Loggable<Self::Fail>) {
        if let Err(e) = self {
            logln!(Warning, "{}: {e}", fail_msg.display(&e));
        }
    }

    #[inline]
    fn or_warn(self, fail_msg: impl Loggable<Self::Fail>, default: Self::Pass) -> Self::Pass {
        self.inspect_err(|e| logln!(Warning, "{}: {e}", fail_msg.display(e)))
            .unwrap_or(default)
    }

    #[inline]
    fn or_warn_with(
        self,
        fail_msg: impl Loggable<Self::Fail>,
        default: impl FnOnce(Self::Fail) -> Self::Pass,
    ) -> Self::Pass {
        self.inspect_err(|e| logln!(Warning, "{}: {e}", fail_msg.display(e)))
            .unwrap_or_else(default)
    }

    #[inline]
    fn error(self, fail_msg: impl Loggable<Self::Fail>) -> Self {
        self.inspect_err(|e| logln!(Error, "{}: {e}", fail_msg.display(e)))
    }

    #[inline]
    fn issue_error(self, fail_msg: impl Loggable<Self::Fail>) {
        if let Err(e) = self {
            logln!(Error, "{}: {e}", fail_msg.display(&e));
        }
    }

    #[inline]
    fn fatal(self, fail_msg: impl Loggable<Self::Fail>) -> Self::Pass {
        self.unwrap_or_else(|e| {
            let msg = fail_msg.display(&e);
            logln!(Error, "{msg}: {e}");
            panic!("fatal error: {msg}: {e}");
        })
    }
}

impl<T> AttemptOption for Option<T> {
    #[inline]
    fn some_info(self, pass_msg: impl Loggable<Self::Pass>) -> Self {
        self.inspect(|v| logln!(Info, "{}.", pass_msg.display(v)))
    }

    #[inline]
    fn none_info(self, pass_msg: impl Loggable<()>) -> Self {
        if self.is_none() {
            logln!(Info, "{}.", pass_msg.display(&()));
        }
        self
    }

    #[inline]
    fn info(self, msg: impl Loggable<Self>) -> Self {
        logln!(Info, "{}", msg.display(&self));
        self
    }

    #[inline]
    fn success(self, pass_msg: impl Loggable<Self::Pass>) -> Self {
        self.inspect(|v| logln!(Success, "{}.", pass_msg.display(v)))
    }

    #[inline]
    fn warn(self, fail_msg: impl Loggable<()>) -> Self {
        if self.is_none() {
            logln!(Warning, "{}.", fail_msg.display(&()));
        }
        self
    }

    #[inline]
    fn issue_warning(self, fail_msg: impl Loggable<()>) {
        if self.is_none() {
            logln!(Warning, "{}.", fail_msg.display(&()));
        }
    }

    #[inline]
    fn or_warn(self, fail_msg: impl Loggable<()>, default: Self::Pass) -> Self::Pass {
        self.warn(fail_msg).unwrap_or(default)
    }

    #[inline]
    fn or_warn_with(
        self,
        fail_msg: impl Loggable<()>,
        default: impl FnOnce() -> Self::Pass,
    ) -> Self::Pass {
        self.warn(fail_msg).unwrap_or_else(default)
    }

    #[inline]
    fn error(self, fail_msg: impl Loggable<()>) -> Self {
        if self.is_none() {
            logln!(Error, "{}.", fail_msg.display(&()));
        }
        self
    }

    #[inline]
    fn issue_error(self, fail_msg: impl Loggable<()>) {
        if self.is_none() {
            logln!(Error, "{}.", fail_msg.display(&()));
        }
    }

    #[inline]
    fn fatal(self, fail_msg: impl Loggable<()>) -> Self::Pass {
        self.unwrap_or_else(|| {
            let msg = fail_msg.display(&());
            logln!(Error, "{msg}.");
            panic!("fatal error: {msg}.");
        })
    }
}
